use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{ActiveView, App};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(f.size());

    // Title / Header
    let header = Paragraph::new(Line::from(vec![
        Span::styled(" Anime-DL-RS ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("| Type /settings or press 's' for settings"),
    ]))
    .block(Block::default().borders(Borders::ALL).title("Navigation"));
    f.render_widget(header, chunks[0]);

    // Main Content based on View
    match app.current_view {
        ActiveView::Search | ActiveView::AnimeList => {
            draw_search_view(f, app, chunks[1]);
        }
        ActiveView::Episodes => {
            draw_episodes_view(f, app, chunks[1]);
        }
        ActiveView::Settings => {
            draw_search_view(f, app, chunks[1]);
            draw_settings_modal(f, app, f.size());
        }
    }

    // Status / Footer
    let footer = Paragraph::new(app.status_message.as_str())
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(footer, chunks[2]);
}

fn draw_search_view(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);

    let search_block = Paragraph::new(app.search_query.as_str())
        .block(Block::default().borders(Borders::ALL).title("Search Anime (Type query & press Enter)"))
        .style(Style::default().fg(Color::Yellow));
    f.render_widget(search_block, chunks[0]);

    let items: Vec<ListItem> = app
        .search_results
        .iter()
        .enumerate()
        .map(|(i, anime)| {
            let lang_str = format!("[{:?}]", anime.language);
            let content = format!("{} {}", lang_str, anime.title);
            let style = if i == app.selected_anime_index {
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Results (Select & press Enter)"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(list, chunks[1]);
}

fn draw_episodes_view(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .episodes
        .iter()
        .enumerate()
        .map(|(i, ep)| {
            let style = if i == app.selected_episode_index {
                Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(ep.title.clone()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Episodes (Press Enter to Download)"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));
    f.render_widget(list, area);
}

fn draw_settings_modal(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(60, 60, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title("Settings (/settings) - Use Up/Down & Space/Enter")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));
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

    let lang_text = format!("Default Language: {:?} (Press 'l' to toggle)", app.config.default_language);
    let lang_para = Paragraph::new(lang_text).block(Block::default().borders(Borders::ALL));
    f.render_widget(lang_para, chunks[0]);

    let players: Vec<ListItem> = app
        .config
        .preferred_players
        .iter()
        .enumerate()
        .map(|(i, player)| {
            let prefix = if i == app.settings_cursor { "> [x] " } else { "  [ ] " };
            let content = format!("{}{}", prefix, player);
            ListItem::new(content)
        })
        .collect();

    let players_list = List::new(players)
        .block(Block::default().borders(Borders::ALL).title("Preferred Players Priority (Use +/- to reorder)"))
        .style(Style::default().fg(Color::White));
    f.render_widget(players_list, chunks[1]);

    let footer = Paragraph::new("Press Esc or /settings to close");
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
