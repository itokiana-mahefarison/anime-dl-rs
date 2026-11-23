use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph, Tabs},
    Frame,
};

use crate::app::{ActiveView, App};

const ASCII_BANNER: &str = r#"
               _                     _ _                
  __ _ _ _  (_)_ __  ___ ___ __| | |___ _ _ ___ 
 / _` | ' \ | | '  \/ -_)___/ _` | |___| '_(_-< 
 \__,_|_||_||_|_|_|_\___|   \__,_|_|   |_| /__/ 
"#;

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();

    // Outer container block with Binsider style border
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Line::from(vec![
            Span::raw("─┤ "),
            Span::styled("anime-dl-rs v0.1.0", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" ├──"),
        ]))
        .title_alignment(Alignment::Center);

    let inner_area = outer_block.inner(size);
    f.render_widget(outer_block, size);

    // Layout inside outer frame: Header (3), Main Body (Min), Optional Download (3), Footer (1)
    let has_download = app.current_download.is_some();
    let constraints = if has_download {
        vec![
            Constraint::Length(3), // Top header / Tabs
            Constraint::Min(8),    // Main workspace
            Constraint::Length(3), // Download Progress Gauge
            Constraint::Length(1), // Keybindings Footer
        ]
    } else {
        vec![
            Constraint::Length(3), // Top header / Tabs
            Constraint::Min(8),    // Main workspace
            Constraint::Length(1), // Keybindings Footer
        ]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);

    // 1. TOP HEADER & TABS (Binsider Style)
    draw_header_tabs(f, app, chunks[0]);

    // 2. MAIN BODY
    match app.current_view {
        ActiveView::Search | ActiveView::AnimeList => {
            draw_search_view(f, app, chunks[1]);
        }
        ActiveView::Seasons => {
            draw_seasons_view(f, app, chunks[1]);
        }
        ActiveView::Episodes => {
            draw_episodes_view(f, app, chunks[1]);
        }
        ActiveView::Settings => {
            draw_settings_view(f, app, chunks[1]);
        }
    }

    // 3. DOWNLOAD GAUGE (if active)
    if has_download {
        if let Some(ref dl) = app.current_download {
            draw_download_bar(f, dl, chunks[2]);
        }
        draw_footer_keybindings(f, app, chunks[3]);
    } else {
        draw_footer_keybindings(f, app, chunks[2]);
    }
}

fn draw_header_tabs(f: &mut Frame, app: &App, area: Rect) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(35), Constraint::Length(38)])
        .split(area);

    let tab_titles = vec!["[1] Search", "[2] Seasons", "[3] Episodes", "[4] Settings"];
    let selected_tab = match app.current_view {
        ActiveView::Search | ActiveView::AnimeList => 0,
        ActiveView::Seasons => 1,
        ActiveView::Episodes => 2,
        ActiveView::Settings => 3,
    };

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)))
        .select(selected_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        )
        .divider(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
    f.render_widget(tabs, header_chunks[0]);

    let provider_name = match app.config.provider {
        crate::config::ProviderType::VoirAnime => "voir-anime",
        crate::config::ProviderType::AnimeSama => "anime-sama",
    };
    let lang_str = format!("{:?}", app.config.default_language);

    let meta_info = Line::from(vec![
        Span::styled("● ", Style::default().fg(Color::Green)),
        Span::styled(provider_name, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::raw(" │ "),
        Span::styled(lang_str, Style::default().fg(Color::Yellow)),
        Span::raw(" │ "),
        Span::styled("./downloads", Style::default().fg(Color::Gray)),
    ]);

    let meta_widget = Paragraph::new(meta_info)
        .alignment(Alignment::Right)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(meta_widget, header_chunks[1]);
}

fn draw_search_view(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8)])
        .split(area);

    let is_search_editing = app.current_view == ActiveView::Search;

    // Search bar with Binsider prompt style
    let search_border_color = if is_search_editing {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let search_title = if is_search_editing {
        "── Search Anime (Type query & press Enter) ──"
    } else {
        "── Search Anime (Press Tab or '/' to Edit) ──"
    };

    let prompt_prefix = Span::styled(" ❯ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
    let query_span = Span::styled(
        &app.search_query,
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    );
    let cursor_span = if is_search_editing {
        Span::styled("█", Style::default().fg(Color::Cyan).add_modifier(Modifier::SLOW_BLINK))
    } else {
        Span::raw("")
    };

    let search_para = Paragraph::new(Line::from(vec![prompt_prefix, query_span, cursor_span]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(search_border_color))
                .title(search_title),
        );
    f.render_widget(search_para, chunks[0]);

    // If search results are empty, render Binsider style welcome / info card
    if app.search_results.is_empty() {
        draw_welcome_card(f, app, chunks[1]);
    } else {
        // Split view: Left = Results List (45%), Right = Details Bento Box (55%)
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(chunks[1]);

        let is_list_active = app.current_view == ActiveView::AnimeList;
        let list_border_color = if is_list_active {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let items: Vec<ListItem> = app
            .search_results
            .iter()
            .enumerate()
            .map(|(i, anime)| {
                let is_selected = i == app.selected_anime_index;
                let lang_color = match anime.language {
                    crate::config::Language::VOSTFR => Color::Cyan,
                    crate::config::Language::VF => Color::LightMagenta,
                };
                let lang_badge = format!("[{:?}]", anime.language);

                let prefix = if is_selected && is_list_active {
                    "▶ "
                } else if is_selected {
                    "▷ "
                } else {
                    "  "
                };

                let line = Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{:<8} ", lang_badge), Style::default().fg(lang_color)),
                    Span::raw(&anime.title),
                ]);

                let style = if is_selected && is_list_active {
                    Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(line).style(style)
            })
            .collect();

        let list_title = format!("── Results ({}) ──", app.search_results.len());
        let results_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(list_border_color))
                    .title(list_title),
            );
        f.render_widget(results_list, body_chunks[0]);

        // Right details card
        draw_anime_preview(f, app, body_chunks[1]);
    }
}

fn draw_welcome_card(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Banner
            Constraint::Length(8), // Info box
            Constraint::Min(2),
        ])
        .margin(1)
        .split(area);

    let banner = Paragraph::new(ASCII_BANNER)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    f.render_widget(banner, chunks[0]);

    // Centered metadata box
    let info_box = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(chunks[1]);

    let provider_name = match app.config.provider {
        crate::config::ProviderType::VoirAnime => "voir-anime.to (WordPress)",
        crate::config::ProviderType::AnimeSama => "anime-sama.to (Single-Page SPA)",
    };

    let text = vec![
        Line::from(vec![
            Span::styled("Provider   : ", Style::default().fg(Color::Cyan)),
            Span::styled(provider_name, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Language   : ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{:?}", app.config.default_language), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Players    : ", Style::default().fg(Color::Cyan)),
            Span::styled(app.config.current_preferred_players().join(" ➔ "), Style::default().fg(Color::LightMagenta)),
        ]),
        Line::from(vec![
            Span::styled("Backend    : ", Style::default().fg(Color::Cyan)),
            Span::styled("yt-dlp (Universal Stream Extraction)", Style::default().fg(Color::Green)),
        ]),
    ];

    let info_block = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title("── Configuration ──")
                .title_alignment(Alignment::Center),
        );
    f.render_widget(info_block, info_box[1]);
}

fn draw_anime_preview(f: &mut Frame, app: &App, area: Rect) {
    if app.search_results.is_empty() {
        return;
    }

    let anime = &app.search_results[app.selected_anime_index];

    let details = vec![
        Line::from(vec![
            Span::styled("Title     : ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(&anime.title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("Language  : ", Style::default().fg(Color::Cyan)),
            Span::styled(format!("{:?}", anime.language), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Target URL: ", Style::default().fg(Color::Cyan)),
            Span::styled(&anime.url, Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("Action    : ", Style::default().fg(Color::Cyan)),
            Span::styled("Press Enter to select season / episodes", Style::default().fg(Color::Green)),
        ]),
    ];

    let preview_box = Paragraph::new(details)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title("── Anime Information ──"),
        );
    f.render_widget(preview_box, area);
}

fn draw_seasons_view(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    let anime_title = if !app.search_results.is_empty() && app.selected_anime_index < app.search_results.len() {
        &app.search_results[app.selected_anime_index].title
    } else {
        "Anime"
    };

    let items: Vec<ListItem> = app
        .seasons
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let is_selected = i == app.selected_season_index;
            let prefix = if is_selected { "▶ " } else { "  " };

            let content = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:02} │ ", i + 1), Style::default().fg(Color::DarkGray)),
                Span::raw(&s.name),
            ]);

            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let list_title = format!("── Saisons ({}) : {} ──", app.seasons.len(), anime_title);
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
                .title(list_title),
        );
    f.render_widget(list, chunks[0]);

    if !app.seasons.is_empty() && app.selected_season_index < app.seasons.len() {
        let season = &app.seasons[app.selected_season_index];
        let details = vec![
            Line::from(vec![
                Span::styled("Anime   : ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(anime_title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Saison  : ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(&season.name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("URL     : ", Style::default().fg(Color::Cyan)),
                Span::styled(&season.url, Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Action  : ", Style::default().fg(Color::DarkGray)),
                Span::styled("Appuyez sur [Entrée] pour charger les épisodes de cette saison", Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Retour  : ", Style::default().fg(Color::DarkGray)),
                Span::styled("Appuyez sur [Échap] ou [q] pour revenir à la liste des animés", Style::default().fg(Color::DarkGray)),
            ]),
        ];

        let season_box = Paragraph::new(details)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title("── Détails de la Saison ──"),
            );
        f.render_widget(season_box, chunks[1]);
    }
}

fn draw_episodes_view(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    let items: Vec<ListItem> = app
        .episodes
        .iter()
        .enumerate()
        .map(|(i, ep)| {
            let is_selected = i == app.selected_episode_index;
            let prefix = if is_selected { "▶ " } else { "  " };

            let content = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:02} │ ", ep.number), Style::default().fg(Color::DarkGray)),
                Span::raw(&ep.title),
            ]);

            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let list_title = format!("── Episodes ({}) ──", app.episodes.len());
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green))
                .title(list_title),
        );
    f.render_widget(list, chunks[0]);

    // Right details box
    if !app.episodes.is_empty() {
        let ep = &app.episodes[app.selected_episode_index];
        let ep_details = vec![
            Line::from(vec![
                Span::styled("Episode   : ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(format!("#{:02} - {}", ep.number, ep.title), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("Player    : ", Style::default().fg(Color::Green)),
                Span::styled(app.config.current_preferred_players().join(" > "), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("File Out  : ", Style::default().fg(Color::Green)),
                Span::styled(format!("./downloads/episode_{:02}.mp4", ep.number), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Instruction: ", Style::default().fg(Color::DarkGray)),
                Span::styled("Press [Enter] to download | Press [a] to download ALL", Style::default().fg(Color::Green)),
            ]),
        ];

        let ep_box = Paragraph::new(ep_details)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::DarkGray))
                    .title("── Episode Details ──"),
            );
        f.render_widget(ep_box, chunks[1]);
    }
}

fn draw_settings_view(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(65, 75, area);
    f.render_widget(Clear, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(6),
        ])
        .margin(1)
        .split(popup_area);

    let main_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .title("── Settings ──")
        .title_alignment(Alignment::Center);
    f.render_widget(main_block, popup_area);

    // Provider row
    let provider_name = match app.config.provider {
        crate::config::ProviderType::VoirAnime => "voir-anime",
        crate::config::ProviderType::AnimeSama => "anime-sama",
    };
    let prov_para = Paragraph::new(Line::from(vec![
        Span::styled("Active Provider: ", Style::default().fg(Color::Cyan)),
        Span::styled(provider_name, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled(" (Press 'p' or Ctrl+P to switch)", Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(prov_para, chunks[0]);

    // Language row
    let lang_para = Paragraph::new(Line::from(vec![
        Span::styled("Default Audio  : ", Style::default().fg(Color::Cyan)),
        Span::styled(format!("{:?}", app.config.default_language), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(" (Press 'l' to toggle VOSTFR / VF)", Style::default().fg(Color::DarkGray)),
    ]))
    .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(lang_para, chunks[1]);

    // Preferred players list
    let current_players = app.config.current_preferred_players();
    let players: Vec<ListItem> = current_players
        .iter()
        .enumerate()
        .map(|(i, player)| {
            let is_cursor = i == app.settings_cursor;
            let prefix = if is_cursor { "▶ [x] " } else { "  [x] " };
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{}. {}", i + 1, player), Style::default().fg(Color::White)),
            ]);
            let style = if is_cursor {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(line).style(style)
        })
        .collect();

    let list_title = format!("── Player Priority: [{}] (Use +/- to reorder) ──", provider_name);
    let players_list = List::new(players)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(list_title),
        );
    f.render_widget(players_list, chunks[2]);
}

fn draw_download_bar(f: &mut Frame, dl: &crate::app::DownloadProgress, area: Rect) {
    let label = format!("{} ➔ {:.1}% [{}]", dl.title, dl.percent, dl.status);
    let gauge = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green))
                .title("── Download Progress ──"),
        )
        .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray))
        .percent(dl.percent.clamp(0.0, 100.0) as u16)
        .label(label);
    f.render_widget(gauge, area);
}

fn draw_footer_keybindings(f: &mut Frame, app: &App, area: Rect) {
    let pills = match app.current_view {
        ActiveView::Search => vec![
            key_pill("Enter", "Search"),
            key_pill("Tab/Down", "Browse"),
            key_pill("Ctrl+P", "Provider"),
            key_pill("Ctrl+S", "Settings"),
            key_pill("Esc", "Quit"),
        ],
        ActiveView::AnimeList => vec![
            key_pill("Enter", "Select"),
            key_pill("Up/Down", "Navigate"),
            key_pill("Tab", "Edit Query"),
            key_pill("Ctrl+P", "Provider"),
            key_pill("Ctrl+S", "Settings"),
            key_pill("Esc", "Back"),
        ],
        ActiveView::Seasons => vec![
            key_pill("Enter", "Select Season"),
            key_pill("Up/Down", "Navigate"),
            key_pill("Ctrl+P", "Provider"),
            key_pill("Ctrl+S", "Settings"),
            key_pill("Esc", "Back"),
        ],
        ActiveView::Episodes => vec![
            key_pill("Enter", "Download"),
            key_pill("a", "Download All"),
            key_pill("Up/Down", "Navigate"),
            key_pill("Ctrl+P", "Provider"),
            key_pill("Ctrl+S", "Settings"),
            key_pill("Esc", "Back"),
        ],
        ActiveView::Settings => vec![
            key_pill("Up/Down", "Select"),
            key_pill("+/-", "Reorder Player"),
            key_pill("p", "Provider"),
            key_pill("l", "Toggle Lang"),
            key_pill("Ctrl+S/Esc", "Close"),
        ],
    };

    let pills_len = pills.len();
    let mut line_spans = Vec::new();
    line_spans.push(Span::raw(" "));
    for (i, pill) in pills.into_iter().enumerate() {
        line_spans.extend(pill);
        if i + 1 < pills_len {
            line_spans.push(Span::raw(" "));
        }
    }

    let footer = Paragraph::new(Line::from(line_spans)).alignment(Alignment::Center);
    f.render_widget(footer, area);
}

fn key_pill<'a>(key: &'a str, action: &'a str) -> Vec<Span<'a>> {
    vec![
        Span::styled("[", Style::default().fg(Color::DarkGray)),
        Span::styled(key, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(" → ", Style::default().fg(Color::DarkGray)),
        Span::styled(action, Style::default().fg(Color::White)),
        Span::styled("]", Style::default().fg(Color::DarkGray)),
    ]
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    let popup_layout_x = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1]);

    popup_layout_x[1]
}
