use anime_dl_rs::app::{ActiveView, App, DownloadProgress};
use anime_dl_rs::config;
use anime_dl_rs::downloader::Downloader;
use anime_dl_rs::providers::ProviderEnum;
use anime_dl_rs::ui;

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use indicatif::ProgressBar;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(name = "anime-dl-rs", author, version, about = "High-performance Anime Downloader for voir-anime & anime-sama")]
struct Args {
    #[arg(short, long, help = "Direct anime URL to download")]
    url: Option<String>,

    #[arg(short, long, default_value = "./downloads", help = "Output directory for downloaded episodes")]
    output: String,

    #[arg(short, long, help = "Episode range to download (e.g. '1-12', '5', '5-', '-10')")]
    range: Option<String>,
}

fn parse_range(range_str: &str) -> Option<(usize, usize)> {
    let s = range_str.trim();
    if let Some((start_s, end_s)) = s.split_once('-') {
        let start = if start_s.is_empty() { 1 } else { start_s.parse().ok()? };
        let end = if end_s.is_empty() { usize::MAX } else { end_s.parse().ok()? };
        Some((start, end))
    } else if let Ok(num) = s.parse::<usize>() {
        Some((num, num))
    } else {
        None
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    let app_temp = App::new();

    // If direct URL is provided via CLI
    if let Some(url) = args.url {
        let provider_type = if url.contains("anime-sama") {
            config::ProviderType::AnimeSama
        } else if url.contains("voir-anime") {
            config::ProviderType::VoirAnime
        } else {
            app_temp.config.provider
        };
        let provider = ProviderEnum::new(provider_type);
        println!("Starting direct download mode with provider [{}] for URL: {}", provider.name(), url);
        let downloader = Downloader::new(args.output);

        println!("Fetching episodes...");
        let target_url = if url.contains("anime-sama") && !url.contains("/saison") && !url.contains("/film") && !url.contains("/oav") {
            match provider.get_seasons(&url).await {
                Ok(seasons) if !seasons.is_empty() => {
                    println!("Found {} season(s) for URL. Using: {}", seasons.len(), seasons[0].name);
                    seasons[0].url.clone()
                }
                _ => url.clone(),
            }
        } else {
            url.clone()
        };
        let mut episodes = provider.get_episodes(&target_url).await?;
        println!("Found {} total episodes.", episodes.len());

        // Apply episode range filter if provided
        if let Some(ref range_str) = args.range {
            if let Some((start, end)) = parse_range(range_str) {
                episodes.retain(|ep| ep.number >= start && ep.number <= end);
                println!("Filtered to {} episodes for range '{}' ({} to {}).", episodes.len(), range_str, start, end);
            } else {
                eprintln!("Invalid range format '{}', expected e.g. '1-12' or '5'. Downloading all episodes.", range_str);
            }
        }

        let preferred = app_temp.config.preferred_players_for(provider_type);
        for ep in episodes {
            println!("Processing Episode {}: {}", ep.number, ep.title);
            let iframes = match provider.get_player_iframes(&ep.url, preferred).await {
                Ok(list) => list,
                Err(e) => {
                    eprintln!("Failed to find player iframe for episode {}: {}", ep.number, e);
                    continue;
                }
            };

            let mut downloaded = false;
            for (idx, iframe) in iframes.iter().enumerate() {
                println!("Trying player {}/{}...", idx + 1, iframes.len());
                match provider.resolve_video_url(iframe).await {
                    Ok(video_url) => {
                        println!("Resolved video URL: {}", video_url);
                        let pb = ProgressBar::new(0);
                        match downloader.download_episode(&video_url, ep.number, pb).await {
                            Ok(_) => {
                                downloaded = true;
                                break;
                            }
                            Err(e) => eprintln!("Player failed: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Failed to resolve video stream: {}", e),
                }
            }

            if !downloaded {
                eprintln!("Failed to download episode {} on all available players.", ep.number);
            }
        }
        return Ok(());
    }

    // Otherwise, launch interactive TUI mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let res = run_tui(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error in TUI: {:?}", err);
    }

    Ok(())
}

async fn run_tui<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    let (download_tx, mut download_rx) = mpsc::channel::<DownloadProgress>(64);

    loop {
        // Drain incoming download progress events without blocking
        while let Ok(progress) = download_rx.try_recv() {
            let is_finished = progress.is_finished;
            let err = progress.error.clone();
            let status = progress.status.clone();
            app.current_download = Some(progress);
            if is_finished {
                if let Some(e) = err {
                    app.status_message = format!("Download failed: {}", e);
                } else {
                    app.status_message = status;
                }
            }
        }

        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                // Global shortcut Ctrl + P to toggle provider
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('p') {
                    app.toggle_provider();
                    let prov = app.get_active_provider();
                    app.status_message = format!("Switched provider to: [{}]", prov.name());
                    if app.current_view != ActiveView::Settings {
                        app.search_results.clear();
                        app.current_view = ActiveView::Search;
                    }
                    continue;
                }

                // Global shortcut Ctrl + S to toggle settings modal
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
                    app.toggle_settings();
                    continue;
                }

                let provider = app.get_active_provider();

                match app.current_view {
                    ActiveView::Settings => match key.code {
                        KeyCode::Esc => {
                            app.toggle_settings();
                        }
                        KeyCode::Char('p') => {
                            app.toggle_provider();
                            let prov = app.get_active_provider();
                            app.status_message = format!("Switched provider to: [{}]", prov.name());
                        }
                        KeyCode::Char('l') => {
                            app.config.default_language = match app.config.default_language {
                                config::Language::VOSTFR => config::Language::VF,
                                config::Language::VF => config::Language::VOSTFR,
                            };
                            let _ = app.config.save();
                        }
                        KeyCode::Up => {
                            app.move_settings_cursor_up();
                        }
                        KeyCode::Down => {
                            let max = app.config.current_preferred_players().len().saturating_sub(1);
                            app.move_settings_cursor_down(max);
                        }
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            let i = app.settings_cursor;
                            if i > 0 {
                                app.config.current_preferred_players_mut().swap(i, i - 1);
                                app.settings_cursor -= 1;
                                let _ = app.config.save();
                            }
                        }
                        KeyCode::Char('-') => {
                            let i = app.settings_cursor;
                            let max = app.config.current_preferred_players().len().saturating_sub(1);
                            if i < max {
                                app.config.current_preferred_players_mut().swap(i, i + 1);
                                app.settings_cursor += 1;
                                let _ = app.config.save();
                            }
                        }
                        _ => {}
                    },
                    ActiveView::Search => match key.code {
                        KeyCode::Char(c) => {
                            // Typing 's' or any character works directly in search input
                            app.search_query.push(c);
                        }
                        KeyCode::Backspace => {
                            app.search_query.pop();
                        }
                        KeyCode::Enter => {
                            if !app.search_query.is_empty() {
                                app.status_message = format!("[{}] Searching for '{}'...", provider.name(), app.search_query);
                                terminal.draw(|f| ui::draw(f, app))?;
                                match provider.search(&app.search_query).await {
                                    Ok(results) => {
                                        app.status_message = format!("Found {} results. Press Down/Enter to select.", results.len());
                                        app.search_results = results;
                                        app.selected_anime_index = 0;
                                        if !app.search_results.is_empty() {
                                            app.current_view = ActiveView::AnimeList;
                                        }
                                    }
                                    Err(e) => {
                                        app.status_message = format!("Search failed: {}", e);
                                    }
                                }
                            }
                        }
                        KeyCode::Tab | KeyCode::Down => {
                            if !app.search_results.is_empty() {
                                app.current_view = ActiveView::AnimeList;
                            }
                        }
                        KeyCode::Esc => {
                            break;
                        }
                        _ => {}
                    },
                    ActiveView::AnimeList => match key.code {
                        KeyCode::Char('q') => {
                            break;
                        }
                        KeyCode::Char('1') => {
                            app.current_view = ActiveView::Search;
                        }
                        KeyCode::Char('2') => {
                            if !app.seasons.is_empty() {
                                app.current_view = ActiveView::Seasons;
                            }
                        }
                        KeyCode::Char('3') => {
                            if !app.episodes.is_empty() {
                                app.current_view = ActiveView::Episodes;
                            }
                        }
                        KeyCode::Char('4') => {
                            app.toggle_settings();
                        }
                        KeyCode::Tab | KeyCode::Char('/') => {
                            app.current_view = ActiveView::Search;
                        }
                        KeyCode::Up => {
                            if app.selected_anime_index > 0 {
                                app.selected_anime_index -= 1;
                            } else {
                                app.current_view = ActiveView::Search;
                            }
                        }
                        KeyCode::Down => {
                            if !app.search_results.is_empty() && app.selected_anime_index < app.search_results.len() - 1 {
                                app.selected_anime_index += 1;
                            }
                        }
                        KeyCode::Enter => {
                            if !app.search_results.is_empty() {
                                let selected_anime = &app.search_results[app.selected_anime_index];
                                app.status_message = format!("Recherche des saisons pour '{}'...", selected_anime.title);
                                terminal.draw(|f| ui::draw(f, app))?;

                                match provider.get_seasons(&selected_anime.url).await {
                                    Ok(seasons) => {
                                        if seasons.len() > 1 {
                                            app.seasons = seasons;
                                            app.selected_season_index = 0;
                                            app.current_view = ActiveView::Seasons;
                                            app.status_message = format!("{} saisons trouvées. Choisissez une saison et appuyez sur Entrée.", app.seasons.len());
                                        } else if !seasons.is_empty() {
                                            let season_url = seasons[0].url.clone();
                                            app.seasons = seasons;
                                            app.selected_season_index = 0;
                                            app.status_message = format!("Fetching episodes for '{}'...", selected_anime.title);
                                            terminal.draw(|f| ui::draw(f, app))?;
                                            match provider.get_episodes(&season_url).await {
                                                Ok(eps) => {
                                                    app.episodes = eps;
                                                    app.selected_episode_index = 0;
                                                    app.current_view = ActiveView::Episodes;
                                                    app.status_message = format!("Found {} episodes. Press Enter on an episode to download.", app.episodes.len());
                                                }
                                                Err(e) => {
                                                    app.status_message = format!("Failed to fetch episodes: {}", e);
                                                }
                                            }
                                        } else {
                                            app.status_message = "Aucune saison trouvée pour cet animé.".to_string();
                                        }
                                    }
                                    Err(e) => {
                                        app.status_message = format!("Failed to fetch seasons: {}", e);
                                    }
                                }
                            }
                        }
                        KeyCode::Esc => {
                            app.current_view = ActiveView::Search;
                        }
                        _ => {}
                    },
                    ActiveView::Seasons => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.current_view = ActiveView::AnimeList;
                        }
                        KeyCode::Char('1') => {
                            app.current_view = ActiveView::Search;
                        }
                        KeyCode::Char('3') => {
                            if !app.episodes.is_empty() {
                                app.current_view = ActiveView::Episodes;
                            }
                        }
                        KeyCode::Char('4') => {
                            app.toggle_settings();
                        }
                        KeyCode::Up => {
                            if app.selected_season_index > 0 {
                                app.selected_season_index -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if !app.seasons.is_empty() && app.selected_season_index < app.seasons.len() - 1 {
                                app.selected_season_index += 1;
                            }
                        }
                        KeyCode::Enter => {
                            if !app.seasons.is_empty() {
                                let season = app.seasons[app.selected_season_index].clone();
                                app.status_message = format!("Chargement des épisodes pour '{}'...", season.name);
                                terminal.draw(|f| ui::draw(f, app))?;

                                match provider.get_episodes(&season.url).await {
                                    Ok(eps) => {
                                        app.episodes = eps;
                                        app.selected_episode_index = 0;
                                        app.current_view = ActiveView::Episodes;
                                        app.status_message = format!("{} épisodes trouvés pour {}. Appuyez sur Entrée pour télécharger.", app.episodes.len(), season.name);
                                    }
                                    Err(e) => {
                                        app.status_message = format!("Erreur lors du chargement des épisodes: {}", e);
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    ActiveView::Episodes => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            if app.seasons.len() > 1 {
                                app.current_view = ActiveView::Seasons;
                            } else {
                                app.current_view = ActiveView::AnimeList;
                            }
                        }
                        KeyCode::Char('1') => {
                            app.current_view = ActiveView::Search;
                        }
                        KeyCode::Char('2') => {
                            if !app.seasons.is_empty() {
                                app.current_view = ActiveView::Seasons;
                            } else {
                                app.current_view = ActiveView::AnimeList;
                            }
                        }
                        KeyCode::Char('4') => {
                            app.toggle_settings();
                        }
                        KeyCode::Up => {
                            if app.selected_episode_index > 0 {
                                app.selected_episode_index -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if !app.episodes.is_empty() && app.selected_episode_index < app.episodes.len() - 1 {
                                app.selected_episode_index += 1;
                            }
                        }
                        KeyCode::Enter => {
                            if !app.episodes.is_empty() {
                                let ep = app.episodes[app.selected_episode_index].clone();
                                let provider_type = app.config.provider;
                                let preferred = app.config.preferred_players_for(provider_type).clone();
                                let tx = download_tx.clone();
                                let downloader = Downloader::new("./downloads".to_string());

                                app.status_message = format!("Starting download for Episode {} in background...", ep.number);

                                tokio::spawn(async move {
                                    let prov = ProviderEnum::new(provider_type);
                                    let _ = download_episode_with_fallback(&prov, &downloader, &ep.url, ep.number, &preferred, Some(tx)).await;
                                });
                            }
                        }
                        KeyCode::Char('a') | KeyCode::Char('A') => {
                            if !app.episodes.is_empty() {
                                let episodes_to_download = app.episodes.clone();
                                let provider_type = app.config.provider;
                                let preferred = app.config.preferred_players_for(provider_type).clone();
                                let tx = download_tx.clone();
                                let downloader = Downloader::new("./downloads".to_string());
                                let total = episodes_to_download.len();

                                app.status_message = format!("Téléchargement de toute la liste ({} épisodes) lancé en arrière-plan...", total);

                                tokio::spawn(async move {
                                    let mut success_count = 0;
                                    for (idx, ep) in episodes_to_download.into_iter().enumerate() {
                                        let prov = ProviderEnum::new(provider_type);
                                        let _ = tx.send(DownloadProgress {
                                            ep_number: ep.number,
                                            title: format!("Episode {:02} ({}/{})", ep.number, idx + 1, total),
                                            downloaded_bytes: 0,
                                            total_bytes: 0,
                                            percent: 0.0,
                                            status: format!("Préparation épisode {}/{} (Ep {:02})...", idx + 1, total, ep.number),
                                            is_finished: false,
                                            error: None,
                                        }).await;

                                        if download_episode_with_fallback(&prov, &downloader, &ep.url, ep.number, &preferred, Some(tx.clone())).await.is_ok() {
                                            success_count += 1;
                                        }
                                    }

                                    let _ = tx.send(DownloadProgress {
                                        ep_number: 0,
                                        title: "Batch Download".to_string(),
                                        downloaded_bytes: 0,
                                        total_bytes: 0,
                                        percent: 100.0,
                                        status: format!("Téléchargement terminé : {}/{} épisodes réussis.", success_count, total),
                                        is_finished: true,
                                        error: None,
                                    }).await;
                                });
                            }
                        }
                        _ => {}
                    },
                }
            }
        }
    }
    Ok(())
}

async fn download_episode_with_fallback(
    prov: &ProviderEnum,
    downloader: &Downloader,
    ep_url: &str,
    ep_num: usize,
    preferred_players: &[String],
    tx: Option<mpsc::Sender<DownloadProgress>>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let iframes = prov.get_player_iframes(ep_url, preferred_players).await?;
    let mut last_error: Box<dyn std::error::Error + Send + Sync> = "No player iframe available".into();

    for (idx, iframe) in iframes.iter().enumerate() {
        if let Some(ref sender) = tx {
            let _ = sender.send(DownloadProgress {
                ep_number: ep_num,
                title: format!("Episode {:02}", ep_num),
                downloaded_bytes: 0,
                total_bytes: 0,
                percent: 0.0,
                status: format!("Essai lecteur {}/{}...", idx + 1, iframes.len()),
                is_finished: false,
                error: None,
            }).await;
        }

        match prov.resolve_video_url(iframe).await {
            Ok(video_url) => {
                match downloader.download_episode_with_progress(&video_url, ep_num, tx.clone()).await {
                    Ok(path) => return Ok(path),
                    Err(e) => {
                        last_error = e;
                    }
                }
            }
            Err(e) => {
                last_error = e;
            }
        }
    }

    if let Some(ref sender) = tx {
        let _ = sender.send(DownloadProgress {
            ep_number: ep_num,
            title: format!("Episode {:02}", ep_num),
            downloaded_bytes: 0,
            total_bytes: 0,
            percent: 0.0,
            status: format!("Échec sur tous les lecteurs (ep {}): {}", ep_num, last_error),
            is_finished: true,
            error: Some(last_error.to_string()),
        }).await;
    }

    Err(last_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_range() {
        assert_eq!(parse_range("1-12"), Some((1, 12)));
        assert_eq!(parse_range("5"), Some((5, 5)));
        assert_eq!(parse_range("5-"), Some((5, usize::MAX)));
        assert_eq!(parse_range("-10"), Some((1, 10)));
        assert_eq!(parse_range("invalid"), None);
    }
}
