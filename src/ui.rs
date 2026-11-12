use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Gauge, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{ActiveView, App};
use crate::theme::Theme;

pub fn draw(f: &mut Frame, app: &App) {
    let theme = Theme::mocha();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(if app.download_progress.is_some() || app.is_downloading { 6 } else { 3 }),
        ])
        .split(f.size());

    // Modern Header with Status Tag (MovieBox-Tui Design System)
    let header_title = Line::from(vec![
        Span::styled(" ⚡ ANIME-DL-RS ", theme.highlight),
        Span::styled("│ ", theme.border),
        Span::styled("MovieBox Design System & Download Engine", theme.text_dim),
    ]);

    let header = Paragraph::new(header_title)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.border_focus)
                .title(Span::styled(" Navigation ", theme.title)),
        );
    f.render_widget(header, chunks[0]);

    // Main Content based on View
    match app.current_view {
        ActiveView::Search | ActiveView::AnimeList => {
            draw_search_view(f, app, chunks[1], &theme);
        }
        ActiveView::Episodes => {
            draw_episodes_view(f, app, chunks[1], &theme);
        }
        ActiveView::Settings => {
            draw_search_view(f, app, chunks[1], &theme);
            draw_settings_modal(f, app, f.size(), &theme);
        }
    }

    // Active Download Status Card / Footer (MovieBox-Tui Logic)
    if let Some(pct) = app.download_progress {
        let _download_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(3)])
            .split(chunks[2]);

        let title = app.download_title.as_deref().unwrap_or("Downloading Media");
        let status_text = app.download_status.as_deref().unwrap_or("Processing...");
        let gauge_label = format!("{} — {:.1}% ({})", title, pct, status_text);

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(theme.success)
                    .title(Span::styled(" 📥 Active Download (MovieBox Engine) ", theme.success)),
            )
            .gauge_style(Style::default().fg(Color::Rgb(166, 227, 161)).bg(Color::Rgb(48, 52, 70)))
            .percent(pct.clamp(0.0, 100.0) as u16)
            .label(Span::styled(gauge_label, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
        f.render_widget(gauge, f.size()); // Render over footer area if active
    } else {
        let footer_text = match app.current_view {
            ActiveView::Search => " [Type query / URL & Enter] Search/Load  [↑/↓] Navigate  [/s] Settings  [Esc] Quit ",
            ActiveView::Episodes => " [Enter] Download Episode  [c/x] Cancel Download  [↑/↓] Navigate  [Esc] Back to Search ",
            ActiveView::Settings => " [↑/↓] Navigate  [+/-] Reorder Player Priority  ['l'] Toggle Lang  [Esc] Close ",
            ActiveView::AnimeList => " [Enter] Select  [Esc] Back ",
        };

        let footer = Paragraph::new(Line::from(vec![
            Span::styled(" 💬 ", theme.shortcut),
            Span::styled(app.status_message.as_str(), theme.text),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.border)
                .title(Span::styled(footer_text, theme.text_dim)),
        );
        f.render_widget(footer, chunks[2]);
    }
}

fn draw_search_view(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    if app.search_results.is_empty() && app.search_query.is_empty() {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),
                Constraint::Length(3),
                Constraint::Min(2),
            ])
            .split(area);

        let ascii_art = vec![
            Line::from(Span::styled("   █████╗ ███╗   ██╗██╗███╗   ███╗███████╗     ██████╗ ██╗", theme.highlight)),
            Line::from(Span::styled("  ██╔══██╗████╗  ██║██║████╗ ████║██╔════╝     ██╔══██╗██║", theme.highlight)),
            Line::from(Span::styled("  ███████║██╔██╗ ██║██║██╔████╔██║█████╗       ██║  ██║██║", theme.accent)),
            Line::from(Span::styled("  ██╔══██║██║╚██╗██║██║██║╚██╔╝██║██╔══╝       ██║  ██║██║", theme.accent)),
            Line::from(Span::styled("  ██║  ██║██║ ╚████║██║██║ ╚═╝ ██║███████╗     ██████╔╝███████╗", theme.title)),
            Line::from(Span::styled("  ╚═╝  ╚═╝╚═╝  ╚═══╝╚═╝╚═╝     ╚═╝╚══════╝     ╚═════╝ ╚══════╝", theme.title)),
        ];

        let logo_paragraph = Paragraph::new(ascii_art)
            .alignment(Alignment::Center);
        f.render_widget(logo_paragraph, layout[0]);

        let search_block = Paragraph::new(app.search_query.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(theme.border_focus)
                    .title(Span::styled(" 🔍 Search Anime or Paste voir-anime.to URL ", theme.title)),
            )
            .style(theme.text);
        f.render_widget(search_block, layout[1]);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5)])
            .split(area);

        let search_block = Paragraph::new(app.search_query.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(theme.border_focus)
                    .title(Span::styled(" 🔍 Search Anime or Paste voir-anime.to URL ", theme.title)),
            )
            .style(theme.text);
        f.render_widget(search_block, chunks[0]);

        let items: Vec<ListItem> = app
            .search_results
            .iter()
            .enumerate()
            .map(|(i, anime)| {
                let lang_color = match anime.language {
                    crate::config::Language::VOSTFR => theme.accent,
                    crate::config::Language::VF => theme.header,
                };
                let lang_str = format!(" [{:?}] ", anime.language);
                let content = Line::from(vec![
                    Span::styled(lang_str, lang_color.add_modifier(Modifier::BOLD)),
                    Span::styled(&anime.title, theme.text),
                ]);

                let style = if i == app.selected_anime_index {
                    Style::default().bg(Color::Rgb(73, 76, 94)).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(content).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(theme.border)
                    .title(Span::styled(" Results (Press Enter to view episodes) ", theme.highlight)),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));
        f.render_widget(list, chunks[1]);
    }
}

fn draw_episodes_view(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let items: Vec<ListItem> = app
        .episodes
        .iter()
        .enumerate()
        .map(|(i, ep)| {
            let content = Line::from(vec![
                Span::styled(format!(" Episode {:02}: ", ep.number), theme.success.add_modifier(Modifier::BOLD)),
                Span::styled(&ep.title, theme.text),
            ]);

            let style = if i == app.selected_episode_index {
                Style::default().bg(Color::Rgb(73, 76, 94)).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.success)
                .title(Span::styled(" 🎬 Episodes (Press Enter to Download) ", theme.success.add_modifier(Modifier::BOLD))),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(list, area);
}

fn draw_settings_modal(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let popup_area = centered_rect(65, 65, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(Span::styled(" ⚙️ Settings (/settings) ", theme.title))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_focus)
        .style(Style::default().bg(Color::Rgb(30, 30, 46)));
    f.render_widget(block, popup_area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .margin(2)
        .split(popup_area);

    let lang_text = format!(" Default Language: {:?} (Press 'l' to toggle) ", app.config.default_language);
    let lang_para = Paragraph::new(lang_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.border)
                .title(Span::styled(" Language Preference ", theme.shortcut)),
        )
        .style(theme.text);
    f.render_widget(lang_para, chunks[0]);

    let players: Vec<ListItem> = app
        .config
        .preferred_players
        .iter()
        .enumerate()
        .map(|(i, player)| {
            let prefix = if i == app.settings_cursor { " ❯ [✔] " } else { "   [ ] " };
            let content = format!("{}{}", prefix, player);
            let style = if i == app.settings_cursor {
                theme.highlight.add_modifier(Modifier::BOLD)
            } else {
                theme.text_dim
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let players_list = List::new(players)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.border)
                .title(Span::styled(" Preferred Players Priority (Use Up/Down & +/- to reorder) ", theme.highlight)),
        );
    f.render_widget(players_list, chunks[1]);

    let footer = Paragraph::new(" Press Esc or /settings to close ")
        .style(theme.text_dim);
    f.render_widget(footer, chunks[2]);
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
