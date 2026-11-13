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

    pub async fn search(&self, query: &str) -> Result<Vec<AnimeItem>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = Vec::new();
        let query_trimmed = query.trim();

        // 1. Direct URL provided
        if query_trimmed.starts_with("http") {
            let lang = if query_trimmed.contains("/vf") {
                Language::VF
            } else {
                Language::VOSTFR
            };
            let parts: Vec<&str> = query_trimmed.split('/').filter(|s| !s.is_empty()).collect();
            let slug = parts.iter().rev().find(|&&s| s != "vostfr" && s != "vf" && !s.starts_with("saison")).copied().unwrap_or("anime");
            let title = slug.replace('-', " ");

            results.push(AnimeItem {
                title,
                url: query_trimmed.to_string(),
                language: lang,
            });
            return Ok(results);
        }

        // 2. Query against Anime-Sama catalog
        let catalog_url = "https://anime-sama.to/catalogue/";
        if let Ok(resp) = self.client.get(catalog_url).send().await {
            if let Ok(html) = resp.text().await {
                let document = Html::parse_document(&html);
                if let Ok(link_selector) = Selector::parse("a") {
                    let mut seen = HashSet::new();
                    let query_lower = query_trimmed.to_lowercase();

                    for el in document.select(&link_selector) {
                        if let Some(href) = el.value().attr("href") {
                            if href.contains("/catalogue/") && !href.ends_with("/catalogue/") {
                                let title = el.text().collect::<Vec<_>>().join(" ").trim().to_string();
                                if !title.is_empty() && (title.to_lowercase().contains(&query_lower) || query_lower.is_empty()) {
                                    if !seen.contains(href) {
                                        let full_url = if href.starts_with("http") {
                                            href.to_string()
                                        } else {
                                            format!("https://anime-sama.to{}", href)
                                        };
                                        results.push(AnimeItem {
                                            title: title.clone(),
                                            url: full_url.clone(),
                                            language: Language::VOSTFR,
                                        });
                                        seen.insert(full_url);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Fallback / direct slug construction if catalog scraping was empty or blocked by Cloudflare
        if results.is_empty() {
            let slug = query_trimmed.to_lowercase().replace(' ', "-");
            results.push(AnimeItem {
                title: format!("{} (VOSTFR)", query_trimmed),
                url: format!("https://anime-sama.to/catalogue/{}/saison1/vostfr/", slug),
                language: Language::VOSTFR,
            });
            results.push(AnimeItem {
                title: format!("{} (VF)", query_trimmed),
                url: format!("https://anime-sama.to/catalogue/{}/saison1/vf/", slug),
                language: Language::VF,
            });
        }

        Ok(results)
    }

    pub async fn get_episodes(&self, anime_url: &str) -> Result<Vec<EpisodeItem>, Box<dyn std::error::Error + Send + Sync>> {
        let clean_url = anime_url.trim_end_matches('/');
        let target_url = if !clean_url.contains("/saison") && !clean_url.contains("/film") {
            format!("{}/saison1/vostfr/", clean_url)
        } else {
            format!("{}/", clean_url)
        };

        let mut episodes = Vec::new();
        let mut episode_count = 0;

        // Attempt to fetch season page HTML
        if let Ok(resp) = self.client.get(&target_url).send().await {
            if let Ok(html) = resp.text().await {
                let document = Html::parse_document(&html);

                // Strategy A: parse <select id="selectEpisodes"> options
                if let Ok(option_sel) = Selector::parse("select#selectEpisodes option, select.selectEpisodes option") {
                    for opt in document.select(&option_sel) {
                        let text = opt.text().collect::<Vec<_>>().join(" ").trim().to_string();
                        if let Some(num) = self.extract_episode_number(&text) {
                            episodes.push(EpisodeItem {
                                number: num,
                                title: text,
                                url: format!("{}#ep{}", target_url, num),
                            });
                        }
                    }
                }

                // Strategy B: parse buttons with description="Episode X"
                if episodes.is_empty() {
                    let re_btn = regex::Regex::new(r#"description=['"]Episode\s*(\d+)['"]"#).unwrap();
                    for caps in re_btn.captures_iter(&html) {
                        if let Ok(num) = caps[1].parse::<usize>() {
                            if num > episode_count {
                                episode_count = num;
                            }
                        }
                    }
                }

                // Strategy C: regex search for eps1 array in scripts
                if episodes.is_empty() && episode_count == 0 {
                    let re_eps = regex::Regex::new(r#"var\s+eps\d+\s*=\s*\[(.*?)\]"#).unwrap();
                    if let Some(caps) = re_eps.captures(&html) {
                        let count = caps[1].split(',').count();
                        if count > 0 {
                            episode_count = count;
                        }
                    }
                }
            }
        }

        if episodes.is_empty() {
            let total = if episode_count > 0 { episode_count } else { 24 };
            for i in 1..=total {
                episodes.push(EpisodeItem {
                    number: i,
                    title: format!("Episode {:02}", i),
                    url: format!("{}#ep{}", target_url, i),
                });
            }
        }

        episodes.sort_by_key(|e| e.number);
        episodes.dedup_by_key(|e| e.number);

        Ok(episodes)
    }

    fn extract_episode_number(&self, text: &str) -> Option<usize> {
        let re = regex::Regex::new(r"(?i)(?:ep|episode)\s*(\d+)").ok()?;
        if let Some(caps) = re.captures(text) {
            return caps.get(1)?.as_str().parse().ok();
        }
        let digits: Vec<usize> = text
            .split(|c: char| !c.is_digit(10))
            .filter_map(|s| s.parse().ok())
            .collect();
        digits.last().copied()
    }

    pub async fn get_player_iframe(&self, episode_url: &str, preferred_players: &[String]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = episode_url.split('#').next().unwrap_or(episode_url);
        
        // Fetch page HTML to check for playerDF iframe or scripts
        if let Ok(resp) = self.client.get(base_url).send().await {
            if let Ok(html) = resp.text().await {
                let document = Html::parse_document(&html);

                // Check iframe#playerDF
                if let Ok(iframe_sel) = Selector::parse("iframe#playerDF, iframe.playerDF, iframe") {
                    for iframe in document.select(&iframe_sel) {
                        if let Some(src) = iframe.value().attr("src") {
                            if !src.is_empty() && !src.contains("cadre") && !src.contains("adcash") && !src.contains("btag") {
                                return Ok(src.to_string());
                            }
                        }
                    }
                }

                // Look for embed links inside HTML scripts matching preferred players
                for pref in preferred_players {
                    let pref_lower = pref.to_lowercase();
                    let re_pattern = format!(r#"["'](https?://[^"']*{}[^"']*)["']"#, pref_lower);
                    if let Ok(re) = regex::Regex::new(&re_pattern) {
                        if let Some(caps) = re.captures(&html) {
                            if let Some(matched) = caps.get(1) {
                                return Ok(matched.as_str().to_string());
                            }
                        }
                    }
                }

                // General embed matcher
                let re_embed = regex::Regex::new(r#"["'](https?://(?:ansembed\.net|vidmoly\.[a-z]+|uqload\.[a-z]+|voe\.sx)[^"']*)["']"#).unwrap();
                if let Some(caps) = re_embed.captures(&html) {
                    if let Some(matched) = caps.get(1) {
                        return Ok(matched.as_str().to_string());
                    }
                }
            }
        }

        // Fallback: return episode_url for yt-dlp to handle
        Ok(episode_url.to_string())
    }

    pub async fn resolve_video_url(&self, iframe_url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let clean_url = iframe_url.split('#').next().unwrap_or(iframe_url);
        
        // If it's already an embed player recognized by yt-dlp, return directly
        if downloader_is_embed(clean_url) {
            return Ok(clean_url.to_string());
        }

        // Otherwise try resolving video source tag
        if let Ok(resp) = self.client.get(clean_url).send().await {
            if let Ok(text) = resp.text().await {
                let document = Html::parse_document(&text);
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
            }
        }

        Ok(clean_url.to_string())
    }
}

fn downloader_is_embed(url: &str) -> bool {
    let lower = url.to_lowercase();
    lower.contains("vidmoly")
        || lower.contains("ansembed")
        || lower.contains("uqload")
        || lower.contains("voe")
        || lower.contains("streamtape")
        || lower.contains("sibnet")
}
