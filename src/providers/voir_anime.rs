use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;

use crate::config::Language;
use super::{AnimeItem, EpisodeItem};

pub struct VoirAnime {
    client: Client,
}

impl VoirAnime {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_default();
        Self { client }
    }

    pub fn name(&self) -> &str {
        "voir-anime"
    }

    pub fn parse_search_html(&self, html: &str) -> Vec<AnimeItem> {
        let document = Html::parse_document(html);
        let link_selector = Selector::parse("a").unwrap();

        let mut results = Vec::new();
        let mut seen_urls = HashSet::new();

        for element in document.select(&link_selector) {
            if let Some(href) = element.value().attr("href") {
                if href.contains("/anime/") && !seen_urls.contains(href) {
                    let title = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    if !title.is_empty() && title.len() > 2 {
                        let language = if href.ends_with("-vf/") || href.contains("-vf-") || title.to_uppercase().contains("VF") {
                            Language::VF
                        } else {
                            Language::VOSTFR
                        };

                        results.push(AnimeItem {
                            title,
                            url: href.to_string(),
                            language,
                        });
                        seen_urls.insert(href.to_string());
                    }
                }
            }
        }

        results
    }

    pub fn parse_episodes_html(&self, html: &str) -> Vec<EpisodeItem> {
        let document = Html::parse_document(html);
        let link_selector = Selector::parse("li.wp-manga-chapter > a").unwrap();

        let mut episodes = Vec::new();
        let mut seen = HashSet::new();

        for el in document.select(&link_selector) {
            if let Some(href) = el.value().attr("href") {
                if !seen.contains(href) {
                    let title = el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    let number = self.extract_episode_number(&title).unwrap_or(episodes.len() + 1);
                    episodes.push(EpisodeItem {
                        number,
                        title,
                        url: href.to_string(),
                    });
                    seen.insert(href.to_string());
                }
            }
        }

        episodes.sort_by_key(|e| e.number);
        episodes
    }

    pub fn extract_episode_number(&self, title: &str) -> Option<usize> {
        let re_lang = regex::Regex::new(r"(?i)-(\d+)-(?:vostfr|vf)").unwrap();
        if let Some(caps) = re_lang.captures(title) {
            if let Ok(num) = caps.get(1)?.as_str().parse::<usize>() {
                return Some(num);
            }
        }

        let re = regex::Regex::new(r"(?i)(?:ep|episode|-)\s*(\d+)").ok()?;
        if let Some(caps) = re.captures(title) {
            if let Ok(num) = caps.get(1)?.as_str().parse::<usize>() {
                return Some(num);
            }
        }
        let digits: Vec<usize> = title
            .split(|c: char| !c.is_digit(10))
            .filter_map(|s| s.parse().ok())
            .collect();
        digits.last().copied()
    }

    pub async fn search(&self, query: &str) -> Result<Vec<AnimeItem>, Box<dyn std::error::Error + Send + Sync>> {
        let search_url = format!("https://voir-anime.to/?s={}&post_type=wp-manga", urlencoding::encode(query));
        let resp = self.client.get(&search_url).send().await?.text().await?;
        Ok(self.parse_search_html(&resp))
    }

    pub async fn get_episodes(&self, anime_url: &str) -> Result<Vec<EpisodeItem>, Box<dyn std::error::Error + Send + Sync>> {
        let resp = self.client.get(anime_url).send().await?.text().await?;
        Ok(self.parse_episodes_html(&resp))
    }

    pub async fn get_player_iframe(&self, episode_url: &str, preferred_players: &[String]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        for player in preferred_players {
            let host_param = match player.as_str() {
                "Stape" => "host=LECTEUR%20Stape",
                "VOE" => "host=LECTEUR%20VOE",
                "MOON" => "host=LECTEUR%20MOON",
                "myTV" => "host=LECTEUR%20myTV",
                _ => continue,
            };

            let separator = if episode_url.contains('?') { "&" } else { "?" };
            let target_url = format!("{}{}{}", episode_url, separator, host_param);

            if let Ok(resp) = self.client.get(&target_url).send().await {
                if let Ok(text) = resp.text().await {
                    let document = Html::parse_document(&text);
                    if let Ok(iframe_sel) = Selector::parse("#chapter-video-frame iframe, iframe") {
                        if let Some(iframe) = document.select(&iframe_sel).next() {
                            if let Some(src) = iframe.value().attr("src") {
                                if !src.is_empty() {
                                    return Ok(src.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        Err("Could not find any working player iframe for this episode.".into())
    }

    pub async fn resolve_video_url(&self, iframe_url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let resp = self.client.get(iframe_url)
            .header("Referer", "https://voir-anime.to/")
            .send().await?.text().await?;

        let document = Html::parse_document(&resp);
        if let Ok(source_sel) = Selector::parse("video source, source") {
            if let Some(source) = document.select(&source_sel).next() {
                if let Some(src) = source.value().attr("src") {
                    if !src.is_empty() {
                        let final_url = if src.starts_with("//") {
                            format!("https:{}", src)
                        } else {
                            src.to_string()
                        };
                        return Ok(final_url);
                    }
                }
            }
        }

        let re_streamtape = regex::Regex::new(r#"id=['"]robotlink['"][^>]*>([^<]+)<"#).unwrap();
        if let Some(caps) = re_streamtape.captures(&resp) {
            let path = caps.get(1).map_or("", |m| m.as_str()).trim();
            if !path.is_empty() {
                let direct_url = if path.starts_with("http") {
                    path.to_string()
                } else {
                    format!("https:{}", path)
                };
                return Ok(direct_url);
            }
        }

        let re_media = regex::Regex::new(r#"["'](https?://[^"']+\.(?:mp4|m3u8)[^"']*)["']"#).unwrap();
        if let Some(caps) = re_media.captures(&resp) {
            if let Some(url_match) = caps.get(1) {
                return Ok(url_match.as_str().to_string());
            }
        }

        Ok(iframe_url.to_string())
    }
}
