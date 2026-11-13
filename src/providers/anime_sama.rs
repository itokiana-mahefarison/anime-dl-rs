use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;

use crate::config::Language;
use super::{AnimeItem, EpisodeItem};

pub struct AnimeSama {
    client: Client,
}

impl AnimeSama {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_default();
        Self { client }
    }

    pub fn name(&self) -> &str {
        "anime-sama"
    }

    pub async fn search(&self, query: &str) -> Result<Vec<AnimeItem>, Box<dyn std::error::Error>> {
        let catalog_url = "https://anime-sama.to/catalogue/";
        let resp = self.client.get(catalog_url).send().await?.text().await?;
        let document = Html::parse_document(&resp);
        let link_selector = Selector::parse("a").unwrap();

        let mut results = Vec::new();
        let mut seen = HashSet::new();
        let query_lower = query.to_lowercase();

        for el in document.select(&link_selector) {
            if let Some(href) = el.value().attr("href") {
                if href.contains("/catalogue/") && !href.ends_with("/catalogue/") {
                    let title = el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    if !title.is_empty() && (title.to_lowercase().contains(&query_lower) || query_lower.is_empty()) {
                        if !seen.contains(href) {
                            results.push(AnimeItem {
                                title: title.clone(),
                                url: if href.starts_with("http") { href.to_string() } else { format!("https://anime-sama.to{}", href) },
                                language: Language::VOSTFR,
                            });
                            seen.insert(href.to_string());
                        }
                    }
                }
            }
        }

        if results.is_empty() {
            let clean_query = query.trim().to_lowercase().replace(' ', "");
            let direct_url = format!("https://anime-sama.to/catalogue/{}/saison1/vostfr/", clean_query);
            results.push(AnimeItem {
                title: query.to_string(),
                url: direct_url,
                language: Language::VOSTFR,
            });
        }

        Ok(results)
    }

    pub async fn get_episodes(&self, anime_url: &str) -> Result<Vec<EpisodeItem>, Box<dyn std::error::Error>> {
        let resp = self.client.get(anime_url).send().await?.text().await?;
        let document = Html::parse_document(&resp);
        let link_selector = Selector::parse("a").unwrap();

        let mut episodes = Vec::new();
        let mut seen = HashSet::new();

        for el in document.select(&link_selector) {
            if let Some(href) = el.value().attr("href") {
                if href.contains("episode") || href.contains("ep") || el.text().collect::<Vec<_>>().join("").to_lowercase().contains("episode") {
                    let title = el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    let full_url = if href.starts_with("http") { href.to_string() } else { format!("https://anime-sama.to{}", href) };
                    if !seen.contains(&full_url) {
                        let num = episodes.len() + 1;
                        episodes.push(EpisodeItem {
                            number: num,
                            title: if title.is_empty() { format!("Episode {}", num) } else { title },
                            url: full_url.clone(),
                        });
                        seen.insert(full_url);
                    }
                }
            }
        }

        if episodes.is_empty() {
            for i in 1..=24 {
                episodes.push(EpisodeItem {
                    number: i,
                    title: format!("Episode {}", i),
                    url: format!("{}#ep{}", anime_url, i),
                });
            }
        }

        Ok(episodes)
    }

    pub async fn get_player_iframe(&self, episode_url: &str, _preferred_players: &[String]) -> Result<String, Box<dyn std::error::Error>> {
        Ok(episode_url.to_string())
    }

    pub async fn resolve_video_url(&self, iframe_url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let clean_url = iframe_url.split('#').next().unwrap_or(iframe_url);
        Ok(clean_url.to_string())
    }
}
