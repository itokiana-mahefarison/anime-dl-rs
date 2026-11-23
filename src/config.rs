use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    VOSTFR,
    VF,
}

impl Default for Language {
    fn default() -> Self {
        Language::VOSTFR
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderType {
    VoirAnime,
    AnimeSama,
}

impl Default for ProviderType {
    fn default() -> Self {
        ProviderType::VoirAnime
    }
}

pub fn default_voir_anime_players() -> Vec<String> {
    vec![
        "Stape".to_string(),
        "myTV".to_string(),
        "MOON".to_string(),
        "VOE".to_string(),
    ]
}

pub fn default_anime_sama_players() -> Vec<String> {
    vec![
        "Sibnet".to_string(),
        "Vidmoly".to_string(),
        "VOE".to_string(),
        "Sendvid".to_string(),
        "Uqload".to_string(),
        "OneUpload".to_string(),
        "Streamtape".to_string(),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub default_language: Language,
    pub provider: ProviderType,
    #[serde(default = "default_voir_anime_players")]
    pub voir_anime_players: Vec<String>,
    #[serde(default = "default_anime_sama_players")]
    pub anime_sama_players: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_players: Option<Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_language: Language::VOSTFR,
            provider: ProviderType::VoirAnime,
            voir_anime_players: default_voir_anime_players(),
            anime_sama_players: default_anime_sama_players(),
            preferred_players: None,
        }
    }
}

impl Config {
    pub fn config_path() -> Option<PathBuf> {
        let home = std::env::var("HOME").ok()?;
        let path = PathBuf::from(home).join(".config").join("anime-dl").join("config.json");
        Some(path)
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(data) = fs::read_to_string(&path) {
                    if let Ok(mut config) = serde_json::from_str::<Config>(&data) {
                        // Backwards compatibility migration:
                        // If legacy config had preferred_players, copy into voir_anime_players if not customized
                        if let Some(ref legacy) = config.preferred_players {
                            if config.voir_anime_players == default_voir_anime_players() && !legacy.is_empty() {
                                config.voir_anime_players = legacy.clone();
                            }
                        }
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let data = serde_json::to_string_pretty(self)?;
            fs::write(path, data)?;
        }
        Ok(())
    }

    pub fn preferred_players_for(&self, provider: ProviderType) -> &Vec<String> {
        match provider {
            ProviderType::VoirAnime => &self.voir_anime_players,
            ProviderType::AnimeSama => &self.anime_sama_players,
        }
    }

    pub fn preferred_players_for_mut(&mut self, provider: ProviderType) -> &mut Vec<String> {
        match provider {
            ProviderType::VoirAnime => &mut self.voir_anime_players,
            ProviderType::AnimeSama => &mut self.anime_sama_players,
        }
    }

    pub fn current_preferred_players(&self) -> &Vec<String> {
        self.preferred_players_for(self.provider)
    }

    pub fn current_preferred_players_mut(&mut self) -> &mut Vec<String> {
        self.preferred_players_for_mut(self.provider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.default_language, Language::VOSTFR);
        assert_eq!(config.voir_anime_players.len(), 4);
        assert_eq!(config.voir_anime_players[0], "Stape");
        assert_eq!(config.anime_sama_players.len(), 7);
        assert_eq!(config.anime_sama_players[0], "Sibnet");
        assert_eq!(config.current_preferred_players(), &config.voir_anime_players);
    }

    #[test]
    fn test_provider_player_switching() {
        let mut config = Config::default();
        assert_eq!(config.current_preferred_players()[0], "Stape");

        config.provider = ProviderType::AnimeSama;
        assert_eq!(config.current_preferred_players()[0], "Sibnet");

        config.current_preferred_players_mut()[0] = "VOE".to_string();
        assert_eq!(config.anime_sama_players[0], "VOE");
        // voir_anime_players must remain untouched
        assert_eq!(config.voir_anime_players[0], "Stape");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            default_language: Language::VF,
            provider: ProviderType::AnimeSama,
            voir_anime_players: vec!["VOE".to_string(), "MOON".to_string()],
            anime_sama_players: vec!["Vidmoly".to_string(), "Sibnet".to_string()],
            preferred_players: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_legacy_config_deserialization() {
        let legacy_json = r#"{
            "default_language": "VOSTFR",
            "provider": "VoirAnime",
            "preferred_players": ["CustomStape", "CustomVOE"]
        }"#;

        let mut config: Config = serde_json::from_str(legacy_json).unwrap();
        if let Some(ref legacy) = config.preferred_players {
            if config.voir_anime_players == default_voir_anime_players() {
                config.voir_anime_players = legacy.clone();
            }
        }

        assert_eq!(config.voir_anime_players, vec!["CustomStape", "CustomVOE"]);
        assert_eq!(config.anime_sama_players, default_anime_sama_players());
    }
}
