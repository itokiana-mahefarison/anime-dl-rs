use anime_dl_rs::app::{ActiveView, App};
use anime_dl_rs::config;
use anime_dl_rs::downloader::Downloader;
use anime_dl_rs::scraper::Scraper;
use anime_dl_rs::ui;

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use indicatif::ProgressBar;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

#[derive(Parser, Debug)]
#[command(name = "anime-dl-rs", author, version, about = "High-performance Anime Downloader for voir-anime.to")]
struct Args {
    #[arg(short, long)]
    url: Option<String>,

    #[arg(short, long, default_value = "./downloads")]
    output: String,

    #[arg(long, help = "Run non-interactive CLI batch download for all episodes")]
    cli: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let scraper = Scraper::new();

    // If --cli flag is passed, run non-interactive batch download mode
    if args.cli {
        if let Some(url) = args.url {
            println!("Starting non-interactive CLI batch download mode for URL: {}", url);
            let downloader = Downloader::new(args.output);

            println!("Fetching episodes...");
            let episodes = scraper.get_episodes(&url).await?;
            println!("Found {} episodes. Downloading in batches of 3 with player fallback...", episodes.len());

            let config = anime_dl_rs::config::Config::load();
            let preferred_players = config.preferred_players;
            println!("Loaded preferred players: {:?}", preferred_players);

            let chunks: Vec<_> = episodes.chunks(3).collect();
            let total_chunks = chunks.len();

            for (chunk_idx, chunk) in chunks.iter().enumerate() {
                println!("\n=== Starting batch {}/{} (3 episodes max in parallel) ===", chunk_idx + 1, total_chunks);
                let mut set = tokio::task::JoinSet::new();

                for ep in *chunk {
                    let scraper = scraper.clone();
                    let downloader = downloader.clone();
                    let players = preferred_players.clone();
                    let ep = (*ep).clone();

                    set.spawn(async move {
                        println!("▶ Processing Episode {}: {}", ep.number, ep.title);
                        let pb = ProgressBar::new(0);
                        let mut downloaded = false;

                        for player in &players {
                            let iframe = match scraper.get_player_iframe_for_player(&ep.url, player).await {
                                Ok(iframe) => iframe,
                                Err(_) => continue,
                            };

                            let target_url = match scraper.resolve_video_url(&iframe).await {
                                Ok(vid_url) => vid_url,
                                Err(_) => iframe,
                            };

                            let (_tx, rx) = tokio::sync::watch::channel(());
                            match downloader.download_episode(&target_url, ep.number, pb.clone(), rx).await {
                                Ok(_) => {
                                    downloaded = true;
                                    break;
                                }
                                Err(e) => {
                                    eprintln!("⚠️ Player {} failed for Episode {}: {}. Trying next player...", player, ep.number, e);
                                }
                            }
                        }

                        if !downloaded {
                            eprintln!("❌ All players failed for Episode {} ({}).", ep.number, ep.title);
                        }
                    });
                }

                while let Some(res) = set.join_next().await {
                    if let Err(e) = res {
                        eprintln!("Task error in batch: {}", e);
                    }
                }
                println!("=== Batch {} completed ===", chunk_idx + 1);
            }

            return Ok(());
        } else {
            eprintln!("Error: --cli requires --url \"<url>\"");
            return Ok(());
        }
    }

    let mut app = App::new();
    app.config.output_dir = args.output;

    // If direct URL is provided without --cli, fetch episodes and open TUI directly in Episodes view
    if let Some(url) = args.url {
        println!("Fetching episodes from direct URL for TUI mode...");
        match scraper.get_episodes(&url).await {
            Ok(eps) => {
                app.episodes = eps;
                app.current_view = ActiveView::Episodes;
                app.status_message = format!("Loaded {} episodes from URL. Press Enter to download.", app.episodes.len());
            }
            Err(e) => {
                eprintln!("Failed to fetch episodes from URL: {}. Starting standard TUI search...", e);
            }
        }
    }

    // Launch interactive TUI mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_tui(&mut terminal, &mut app, &scraper).await;

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
    scraper: &Scraper,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.current_view {
                    ActiveView::Settings => match key.code {
                        KeyCode::Esc | KeyCode::Char('s') => {
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
                            app.search_query.push(c);
                        }
                        KeyCode::Backspace => {
                            app.search_query.pop();
                        }
                        KeyCode::Enter => {
                            let query = app.search_query.trim().to_string();
                            if query == "/s" || query == "/settings" {
                                app.toggle_settings();
                                app.search_query.clear();
                            } else if query.starts_with("http://") || query.starts_with("https://") || query.contains("voir-anime.to") {
                                app.status_message = format!("Fetching episodes from URL '{}'...", query);
                                match scraper.get_episodes(&query).await {
                                    Ok(eps) => {
                                        app.episodes = eps;
                                        app.selected_episode_index = 0;
                                        app.current_view = ActiveView::Episodes;
                                        app.status_message = format!("Loaded {} episodes from URL.", app.episodes.len());
                                    }
                                    Err(_) => {
                                        app.status_message = "Failed to fetch episodes from URL.".to_string();
                                    }
                                }
                            } else if !app.search_results.is_empty() {
                                let anime = app.search_results[app.selected_anime_index].clone();
                                app.status_message = format!("Fetching episodes for {}...", anime.title);
                                match scraper.get_episodes(&anime.url).await {
                                    Ok(eps) => {
                                        app.episodes = eps;
                                        app.selected_episode_index = 0;
                                        app.current_view = ActiveView::Episodes;
                                        app.status_message = format!("Loaded {} episodes.", app.episodes.len());
                                    }
                                    Err(_) => {
                                        app.status_message = "Failed to fetch episodes.".to_string();
                                    }
                                }
                            } else if !query.is_empty() {
                                app.status_message = format!("Searching for '{}'...", query);
                                if let Ok(results) = scraper.search(&query).await {
                                    app.search_results = results;
                                    app.selected_anime_index = 0;
                                    app.status_message = format!("Found {} results. Use Up/Down and Enter to select.", app.search_results.len());
                                } else {
                                    app.status_message = "Search failed.".to_string();
                                }
                            }
                        }
                        KeyCode::Up => {
                            if app.selected_anime_index > 0 {
                                app.selected_anime_index -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if !app.search_results.is_empty() && app.selected_anime_index < app.search_results.len() - 1 {
                                app.selected_anime_index += 1;
                            }
                        }
                        KeyCode::Esc => {
                            break;
                        }
                        _ => {}
                    },
                    ActiveView::AnimeList | ActiveView::Episodes => {
                        match key.code {
                            KeyCode::Esc => {
                                app.current_view = ActiveView::Search;
                            }
                            KeyCode::Char('c') | KeyCode::Char('x') => {
                                if app.is_downloading {
                                    if let Some(tx) = app.download_cancel_tx.take() {
                                        let _ = tx.send(());
                                    }
                                    app.is_downloading = false;
                                    app.status_message = "Download cancelled by user.".to_string();
                                }
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
                                    if let Some(tx) = app.download_cancel_tx.take() {
                                        let _ = tx.send(());
                                    }
                                    let (tx, rx) = tokio::sync::watch::channel(());
                                    app.download_cancel_tx = Some(tx);
                                    app.is_downloading = true;

                                    let ep = app.episodes[app.selected_episode_index].clone();
                                    app.status_message = format!("Downloading Episode {}: {}...", ep.number, ep.title);

                                    let scraper = scraper.clone();
                                    let downloader = Downloader::new(app.config.output_dir.clone());
                                    let players = app.config.preferred_players.clone();
                                    let cancel_rx = rx;

                                    tokio::spawn(async move {
                                        let pb = ProgressBar::hidden();
                                        let mut downloaded = false;
                                        for player in &players {
                                            let iframe = match scraper.get_player_iframe_for_player(&ep.url, player).await {
                                                Ok(iframe) => iframe,
                                                Err(_) => continue,
                                            };
                                            let target_url = match scraper.resolve_video_url(&iframe).await {
                                                Ok(vid_url) => vid_url,
                                                Err(_) => iframe,
                                            };
                                            match downloader.download_episode(&target_url, ep.number, pb.clone(), cancel_rx.clone()).await {
                                                Ok(_) => {
                                                    downloaded = true;
                                                    break;
                                                }
                                                Err(e) => {
                                                    if e.to_string().contains("cancelled") {
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                        if downloaded {
                                            println!("✅ Episode {} downloaded successfully!", ep.number);
                                        } else {
                                            println!("❌ Download stopped or failed for Episode {}.", ep.number);
                                        }
                                    });
                                    app.status_message = format!("Started background download for Episode {}. Press 'c' to cancel.", ep.number);
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
