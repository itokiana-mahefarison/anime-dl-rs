use anime_dl_rs::app::{ActiveView, App};
use anime_dl_rs::config;
use anime_dl_rs::downloader::Downloader;
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

#[derive(Parser, Debug)]
#[command(name = "anime-dl-rs", author, version, about = "High-performance Anime Downloader for voir-anime & anime-sama")]
struct Args {
    #[arg(short, long)]
    url: Option<String>,

    #[arg(short, long, default_value = "./downloads")]
    output: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let app_temp = App::new();
    let provider = app_temp.get_active_provider();

    // If direct URL is provided via CLI
    if let Some(url) = args.url {
        println!("Starting direct download mode with provider [{}] for URL: {}", provider.name(), url);
        let downloader = Downloader::new(args.output);

        println!("Fetching episodes...");
        let episodes = provider.get_episodes(&url).await?;
        println!("Found {} episodes.", episodes.len());

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
    let downloader = Downloader::new("./downloads".to_string());

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Global shortcut Ctrl + P to toggle provider
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('p') {
                    app.toggle_provider();
                    let prov = app.get_active_provider();
                    app.status_message = format!("Switched provider to: [{}]", prov.name());
                    app.search_results.clear();
                    continue;
                }

                let provider = app.get_active_provider();

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
                        KeyCode::Char('s') => {
                            app.toggle_settings();
                        }
                        KeyCode::Char(c) => {
                            app.search_query.push(c);
                        }
                        KeyCode::Backspace => {
                            app.search_query.pop();
                        }
                        KeyCode::Enter => {
                            if !app.search_results.is_empty() {
                                // Select anime and fetch episodes
                                let selected_anime = &app.search_results[app.selected_anime_index];
                                app.status_message = format!("Fetching episodes for '{}'...", selected_anime.title);
                                terminal.draw(|f| ui::draw(f, app))?;

                                if let Ok(eps) = provider.get_episodes(&selected_anime.url).await {
                                    app.episodes = eps;
                                    app.selected_episode_index = 0;
                                    app.current_view = ActiveView::Episodes;
                                    app.status_message = format!("Found {} episodes. Press Enter on an episode to download.", app.episodes.len());
                                } else {
                                    app.status_message = "Failed to fetch episodes.".to_string();
                                }
                            } else if !app.search_query.is_empty() {
                                app.status_message = format!("[{}] Searching for '{}'...", provider.name(), app.search_query);
                                terminal.draw(|f| ui::draw(f, app))?;
                                if let Ok(results) = provider.search(&app.search_query).await {
                                    app.search_results = results;
                                    app.selected_anime_index = 0;
                                    app.status_message = format!("Found {} results.", app.search_results.len());
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
                    ActiveView::Episodes => match key.code {
                        KeyCode::Esc => {
                            app.current_view = ActiveView::Search;
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
                                let ep = &app.episodes[app.selected_episode_index];
                                app.status_message = format!("Resolving episode {}: {}...", ep.number, ep.title);
                                terminal.draw(|f| ui::draw(f, app))?;

                                match provider.get_player_iframe(&ep.url, &app.config.preferred_players).await {
                                    Ok(iframe) => {
                                        match provider.resolve_video_url(&iframe).await {
                                            Ok(video_url) => {
                                                app.status_message = format!("Downloading episode {}...", ep.number);
                                                terminal.draw(|f| ui::draw(f, app))?;
                                                let pb = ProgressBar::new(0);
                                                if let Err(e) = downloader.download_episode(&video_url, ep.number, pb).await {
                                                    app.status_message = format!("Download failed: {}", e);
                                                } else {
                                                    app.status_message = format!("Episode {} downloaded successfully!", ep.number);
                                                }
                                            }
                                            Err(e) => {
                                                app.status_message = format!("Failed to resolve stream: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        app.status_message = format!("Failed to find player: {}", e);
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    ActiveView::AnimeList => {
                        if key.code == KeyCode::Esc {
                            app.current_view = ActiveView::Search;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
