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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub default_language: Language,
    pub preferred_players: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_language: Language::VOSTFR,
            preferred_players: vec![
                "Stape".to_string(),
                "myTV".to_string(),
                "MOON".to_string(),
                "VOE".to_string(),
            ],
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
                    if let Ok(config) = serde_json::from_str(&data) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.default_language, Language::VOSTFR);
        assert_eq!(config.preferred_players.len(), 4);
        assert_eq!(config.preferred_players[0], "Stape");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            default_language: Language::VF,
            preferred_players: vec!["VOE".to_string(), "MOON".to_string()],
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(config, deserialized);
    }
}
