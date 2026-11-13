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
        let provider = if url.contains("anime-sama") {
            ProviderEnum::new(config::ProviderType::AnimeSama)
        } else if url.contains("voir-anime") {
            ProviderEnum::new(config::ProviderType::VoirAnime)
        } else {
            app_temp.get_active_provider()
        };
        println!("Starting direct download mode with provider [{}] for URL: {}", provider.name(), url);
        let downloader = Downloader::new(args.output);

        println!("Fetching episodes...");
        let mut episodes = provider.get_episodes(&url).await?;
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

        for ep in episodes {
            println!("Processing Episode {}: {}", ep.number, ep.title);
            match provider.get_player_iframe(&ep.url, &app_temp.config.preferred_players).await {
                Ok(iframe) => {
                    match provider.resolve_video_url(&iframe).await {
                        Ok(video_url) => {
                            println!("Resolved video URL: {}", video_url);
                            let pb = ProgressBar::new(0);
                            if let Err(e) = downloader.download_episode(&video_url, ep.number, pb).await {
                                eprintln!("Failed to download episode {}: {}", ep.number, e);
                            }
                        }
                        Err(e) => eprintln!("Failed to resolve video stream: {}", e),
                    }
                }
                Err(e) => eprintln!("Failed to find player iframe: {}", e),
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
                    app.search_results.clear();
                    app.current_view = ActiveView::Search;
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
                            let max = app.config.preferred_players.len().saturating_sub(1);
                            app.move_settings_cursor_down(max);
                        }
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            let i = app.settings_cursor;
                            if i > 0 {
                                app.config.preferred_players.swap(i, i - 1);
                                app.settings_cursor -= 1;
                                let _ = app.config.save();
                            }
                        }
                        KeyCode::Char('-') => {
                            let i = app.settings_cursor;
                            let max = app.config.preferred_players.len().saturating_sub(1);
                            if i < max {
                                app.config.preferred_players.swap(i, i + 1);
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
                            if !app.episodes.is_empty() {
                                app.current_view = ActiveView::Episodes;
                            }
                        }
                        KeyCode::Char('3') => {
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
                                app.status_message = format!("Fetching episodes for '{}'...", selected_anime.title);
                                terminal.draw(|f| ui::draw(f, app))?;

                                match provider.get_episodes(&selected_anime.url).await {
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
                            }
                        }
                        KeyCode::Esc => {
                            app.current_view = ActiveView::Search;
                        }
                        _ => {}
                    },
                    ActiveView::Episodes => match key.code {
                        KeyCode::Char('q') => {
                            app.current_view = ActiveView::AnimeList;
                        }
                        KeyCode::Char('1') => {
                            app.current_view = ActiveView::Search;
                        }
                        KeyCode::Char('3') => {
                            app.toggle_settings();
                        }
                        KeyCode::Esc => {
                            app.current_view = ActiveView::AnimeList;
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
                                let preferred = app.config.preferred_players.clone();
                                let provider_type = app.config.provider;
                                let tx = download_tx.clone();
                                let downloader = Downloader::new("./downloads".to_string());

                                app.status_message = format!("Starting download for Episode {} in background...", ep.number);

                                tokio::spawn(async move {
                                    let prov = ProviderEnum::new(provider_type);
                                    match prov.get_player_iframe(&ep.url, &preferred).await {
                                        Ok(iframe) => {
                                            match prov.resolve_video_url(&iframe).await {
                                                Ok(video_url) => {
                                                    let _ = downloader.download_episode_with_progress(&video_url, ep.number, Some(tx.clone())).await;
                                                }
                                                Err(e) => {
                                                    let _ = tx.send(DownloadProgress {
                                                        ep_number: ep.number,
                                                        title: format!("Episode {:02}", ep.number),
                                                        downloaded_bytes: 0,
                                                        total_bytes: 0,
                                                        percent: 0.0,
                                                        status: format!("Failed to resolve stream: {}", e),
                                                        is_finished: true,
                                                        error: Some(e.to_string()),
                                                    }).await;
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            let _ = tx.send(DownloadProgress {
                                                ep_number: ep.number,
                                                title: format!("Episode {:02}", ep.number),
                                                downloaded_bytes: 0,
                                                total_bytes: 0,
                                                percent: 0.0,
                                                status: format!("Failed to find player iframe: {}", e),
                                                is_finished: true,
                                                error: Some(e.to_string()),
                                            }).await;
                                        }
                                    }
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
