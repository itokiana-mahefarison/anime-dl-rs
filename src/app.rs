use crate::config::{Config, ProviderType};
use crate::providers::{AnimeItem, EpisodeItem, ProviderEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Search,
    AnimeList,
    Episodes,
    Settings,
}

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub ep_number: usize,
    pub title: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percent: f64,
    pub status: String,
    pub is_finished: bool,
    pub error: Option<String>,
}

pub struct App {
    pub current_view: ActiveView,
    pub previous_view: ActiveView,
    pub config: Config,
    pub search_query: String,
    pub search_results: Vec<AnimeItem>,
    pub selected_anime_index: usize,
    pub episodes: Vec<EpisodeItem>,
    pub selected_episode_index: usize,
    pub status_message: String,
    pub is_loading: bool,
    pub settings_cursor: usize,
    pub current_download: Option<DownloadProgress>,
}

impl App {
    pub fn new() -> Self {
        let config = Config::load();
        Self {
            current_view: ActiveView::Search,
            previous_view: ActiveView::Search,
            config,
            search_query: String::new(),
            search_results: Vec::new(),
            selected_anime_index: 0,
            episodes: Vec::new(),
            selected_episode_index: 0,
            status_message: "Type your search query and press Enter. (Ctrl+P: Provider | Ctrl+S: Settings)".to_string(),
            is_loading: false,
            settings_cursor: 0,
            current_download: None,
        }
    }

    pub fn get_active_provider(&self) -> ProviderEnum {
        ProviderEnum::new(self.config.provider)
    }

    pub fn toggle_provider(&mut self) {
        self.config.provider = match self.config.provider {
            ProviderType::VoirAnime => ProviderType::AnimeSama,
            ProviderType::AnimeSama => ProviderType::VoirAnime,
        };
        let _ = self.config.save();
    }

    pub fn toggle_settings(&mut self) {
        if self.current_view == ActiveView::Settings {
            self.current_view = self.previous_view;
        } else {
            self.previous_view = self.current_view;
            self.current_view = ActiveView::Settings;
        }
    }

    pub fn move_settings_cursor_up(&mut self) {
        if self.settings_cursor > 0 {
            self.settings_cursor -= 1;
        }
    }

    pub fn move_settings_cursor_down(&mut self, max: usize) {
        if self.settings_cursor < max {
            self.settings_cursor += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_initial_state() {
        let app = App::new();
        assert_eq!(app.current_view, ActiveView::Search);
        assert!(app.search_query.is_empty());
        assert!(app.current_download.is_none());
    }

    #[test]
    fn test_toggle_settings() {
        let mut app = App::new();
        assert_eq!(app.current_view, ActiveView::Search);
        
        app.toggle_settings();
        assert_eq!(app.current_view, ActiveView::Settings);

        app.toggle_settings();
        assert_eq!(app.current_view, ActiveView::Search);
    }

    #[test]
    fn test_toggle_provider() {
        let mut app = App::new();
        let initial = app.config.provider;
        app.toggle_provider();
        assert_ne!(app.config.provider, initial);
        app.toggle_provider();
        assert_eq!(app.config.provider, initial);
    }

    #[test]
    fn test_settings_cursor_movement() {
        let mut app = App::new();
        assert_eq!(app.settings_cursor, 0);

        app.move_settings_cursor_down(3);
        assert_eq!(app.settings_cursor, 1);

        app.move_settings_cursor_up();
        assert_eq!(app.settings_cursor, 0);

        // Cannot go below 0
        app.move_settings_cursor_up();
        assert_eq!(app.settings_cursor, 0);
    }
}
