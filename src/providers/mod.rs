pub mod voir_anime;
pub mod anime_sama;

use serde::{Deserialize, Serialize};
use crate::config::{Language, ProviderType};
use self::voir_anime::VoirAnime;
use self::anime_sama::AnimeSama;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeItem {
    pub title: String,
    pub url: String,
    pub language: Language,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeItem {
    pub number: usize,
    pub title: String,
    pub url: String,
}

pub enum ProviderEnum {
    VoirAnime(VoirAnime),
    AnimeSama(AnimeSama),
}

impl ProviderEnum {
    pub fn new(provider_type: ProviderType) -> Self {
        match provider_type {
            ProviderType::VoirAnime => ProviderEnum::VoirAnime(VoirAnime::new()),
            ProviderType::AnimeSama => ProviderEnum::AnimeSama(AnimeSama::new()),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ProviderEnum::VoirAnime(p) => p.name(),
            ProviderEnum::AnimeSama(p) => p.name(),
        }
    }

    pub async fn search(&self, query: &str) -> Result<Vec<AnimeItem>, Box<dyn std::error::Error>> {
        match self {
            ProviderEnum::VoirAnime(p) => p.search(query).await,
            ProviderEnum::AnimeSama(p) => p.search(query).await,
        }
    }

    pub async fn get_episodes(&self, anime_url: &str) -> Result<Vec<EpisodeItem>, Box<dyn std::error::Error>> {
        match self {
            ProviderEnum::VoirAnime(p) => p.get_episodes(anime_url).await,
            ProviderEnum::AnimeSama(p) => p.get_episodes(anime_url).await,
        }
    }

    pub async fn get_player_iframe(&self, episode_url: &str, preferred_players: &[String]) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            ProviderEnum::VoirAnime(p) => p.get_player_iframe(episode_url, preferred_players).await,
            ProviderEnum::AnimeSama(p) => p.get_player_iframe(episode_url, preferred_players).await,
        }
    }

    pub async fn resolve_video_url(&self, iframe_url: &str) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            ProviderEnum::VoirAnime(p) => p.resolve_video_url(iframe_url).await,
            ProviderEnum::AnimeSama(p) => p.resolve_video_url(iframe_url).await,
        }
    }
}
