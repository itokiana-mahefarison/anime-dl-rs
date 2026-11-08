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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // If direct URL is provided via CLI
    if let Some(url) = args.url {
        println!("Starting direct download mode for URL: {}", url);
        let scraper = Scraper::new();
        let downloader = Downloader::new(args.output);

        println!("Fetching episodes...");
        let episodes = scraper.get_episodes(&url).await?;
        println!("Found {} episodes.", episodes.len());

        for ep in episodes {
            println!("Processing Episode {}: {}", ep.number, ep.title);
            match scraper.get_player_iframe(&ep.url, &["Stape".to_string(), "myTV".to_string(), "MOON".to_string(), "VOE".to_string()]).await {
                Ok(iframe) => {
                    println!("Found iframe: {}", iframe);
                    match scraper.resolve_video_url(&iframe).await {
                        Ok(video_url) => {
                            println!("Resolved direct video URL: {}", video_url);
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
    let scraper = Scraper::new();

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
                            if !app.search_query.is_empty() {
                                app.status_message = format!("Searching for '{}'...", app.search_query);
                                if let Ok(results) = scraper.search(&app.search_query).await {
                                    app.search_results = results;
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
                    ActiveView::AnimeList | ActiveView::Episodes => {
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
