use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;

use crate::config::Language;
use super::{AnimeItem, EpisodeItem, SeasonItem};

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

    pub async fn check_vf_available(&self, catalogue_url: &str) -> bool {
        let clean_root = self.get_catalogue_root(catalogue_url);
        let root_url = format!("{}/", clean_root.trim_end_matches('/'));

        // Method 1: Check catalogue main page HTML for VF mentions in "Sur le site" or flags
        if let Ok(resp) = self.client.get(&root_url).send().await {
            if let Ok(html) = resp.text().await {
                let doc = Html::parse_document(&html);
                if let Ok(dl_sel) = Selector::parse("dl.oe-infos, dl") {
                    for dl in doc.select(&dl_sel) {
                        let txt = dl.text().collect::<Vec<_>>().join(" ");
                        if (txt.contains("Sur le site") || txt.contains("épisodes") || txt.contains("épisode")) && txt.contains("VF") {
                            return true;
                        }
                    }
                }
                if let Ok(flag_sel) = Selector::parse(".as-drapeau img") {
                    for img in doc.select(&flag_sel) {
                        let alt = img.value().attr("alt").unwrap_or("");
                        let title = img.value().attr("title").unwrap_or("");
                        if alt.eq_ignore_ascii_case("VF") || title.eq_ignore_ascii_case("VF") {
                            return true;
                        }
                    }
                }
                if html.contains("VOSTFR et VF") || html.contains("en VF") || html.contains("VOSTFR, VF") {
                    return true;
                }
            }
        }

        // Method 2: Fast check if saison1/vf/ exists (returns 200 vs 404)
        let vf_test_url = format!("{}/saison1/vf/", clean_root.trim_end_matches('/'));
        if let Ok(resp) = self.client.get(&vf_test_url).send().await {
            if resp.status().is_success() {
                return true;
            }
        }

        false
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

        // Temporary storage for unique anime discovered: (title, catalogue_url)
        let mut discovered = Vec::new();
        let mut seen_roots = HashSet::new();

        // 2. Query against Anime-Sama via AJAX search endpoint
        let search_url = "https://anime-sama.to/template-php/defaut/fetch.php";
        if let Ok(resp) = self.client
            .post(search_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Referer", "https://anime-sama.to/")
            .header("X-Requested-With", "XMLHttpRequest")
            .body(format!("query={}", urlencoding::encode(query_trimmed)))
            .send()
            .await
        {
            if let Ok(html) = resp.text().await {
                let document = Html::parse_document(&html);
                if let Ok(link_selector) = Selector::parse("a.asn-search-result, a[href*='/catalogue/']") {
                    for el in document.select(&link_selector) {
                        if let Some(href) = el.value().attr("href") {
                            if href.contains("/catalogue/") && !href.contains("?search=") {
                                let title_parts: Vec<&str> = el.text().collect();
                                let title = title_parts.join(" ").trim().to_string();
                                // Extract clean title from h3 if possible
                                let clean_title = if let Ok(h3_sel) = Selector::parse("h3") {
                                    el.select(&h3_sel)
                                        .next()
                                        .map(|h| h.text().collect::<Vec<_>>().join("").trim().to_string())
                                        .filter(|t| !t.is_empty())
                                        .unwrap_or_else(|| title.clone())
                                } else {
                                    title.clone()
                                };
                                if !clean_title.is_empty() {
                                    let full_url = if href.starts_with("http") {
                                        href.to_string()
                                    } else {
                                        format!("https://anime-sama.to{}", href)
                                    };
                                    let catalogue_url = {
                                        let root = self.get_catalogue_root(&full_url);
                                        format!("{}/", root.trim_end_matches('/'))
                                    };
                                    if !seen_roots.contains(&catalogue_url) {
                                        seen_roots.insert(catalogue_url.clone());
                                        discovered.push((clean_title, catalogue_url));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Fallback via catalogue static page if POST search failed
        if discovered.is_empty() {
            let catalog_url = format!("https://anime-sama.to/catalogue/?search={}", urlencoding::encode(query_trimmed));
            if let Ok(resp) = self.client.get(&catalog_url).send().await {
                if let Ok(html) = resp.text().await {
                    let document = Html::parse_document(&html);
                    if let Ok(card_sel) = Selector::parse(".as-carte-oeuvre, a[href*='/catalogue/']") {
                        let query_lower = query_trimmed.to_lowercase();
                        for el in document.select(&card_sel) {
                            let href = el.value().attr("data-url")
                                .or_else(|| el.value().attr("href"))
                                .or_else(|| {
                                    el.select(&Selector::parse("a").ok()?)
                                        .next()?
                                        .value().attr("href")
                                });
                            if let Some(href_str) = href {
                                if href_str.contains("/catalogue/") && !href_str.ends_with("/catalogue/") {
                                    let title = el.value().attr("data-nom")
                                        .map(|n| n.trim().to_string())
                                        .unwrap_or_else(|| {
                                            el.text().collect::<Vec<_>>().join(" ").trim().to_string()
                                        });
                                    if !title.is_empty() && (title.to_lowercase().contains(&query_lower) || query_lower.is_empty()) {
                                        let full_url = if href_str.starts_with("http") {
                                            href_str.to_string()
                                        } else {
                                            format!("https://anime-sama.to{}", href_str)
                                        };
                                        let catalogue_url = {
                                            let root = self.get_catalogue_root(&full_url);
                                            format!("{}/", root.trim_end_matches('/'))
                                        };
                                        if !seen_roots.contains(&catalogue_url) {
                                            seen_roots.insert(catalogue_url.clone());
                                            discovered.push((title, catalogue_url));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 4. Populate VOSTFR and VF items based on availability
        for (title, catalogue_url) in discovered {
            let has_vf = self.check_vf_available(&catalogue_url).await;

            results.push(AnimeItem {
                title: title.clone(),
                url: format!("{}/vostfr", catalogue_url.trim_end_matches('/')),
                language: Language::VOSTFR,
            });

            if has_vf {
                results.push(AnimeItem {
                    title,
                    url: format!("{}/vf", catalogue_url.trim_end_matches('/')),
                    language: Language::VF,
                });
            }
        }

        // 5. Last resort fallback: direct slug construction
        if results.is_empty() {
            let slug = query_trimmed.to_lowercase().replace(' ', "-");
            let base = format!("https://anime-sama.to/catalogue/{}", slug);
            let has_vf = self.check_vf_available(&base).await;

            results.push(AnimeItem {
                title: query_trimmed.to_string(),
                url: format!("{}/vostfr", base),
                language: Language::VOSTFR,
            });
            if has_vf {
                results.push(AnimeItem {
                    title: query_trimmed.to_string(),
                    url: format!("{}/vf", base),
                    language: Language::VF,
                });
            }
        }

        Ok(results)
    }

    pub fn get_catalogue_root(&self, url: &str) -> String {
        let clean = url.trim_end_matches('/');
        if let Some(pos) = clean.find("/catalogue/") {
            let rest = &clean[pos + "/catalogue/".len()..];
            let slug = rest.split('/').next().unwrap_or(rest);
            return format!("https://anime-sama.to/catalogue/{}", slug);
        }
        clean.to_string()
    }

    pub fn parse_seasons_html(&self, html: &str, base_url: &str) -> Vec<SeasonItem> {
        let mut seasons = Vec::new();
        let mut seen_urls = HashSet::new();
        let clean_base = base_url.trim_end_matches('/');

        // Method 1: regex matching panneauAnime("Saison 1", "saison1/vostfr")
        let re_panneau = regex::Regex::new(r#"panneauAnime\s*\(\s*["']([^"']+)["']\s*,\s*["']([^"']+)["']\s*\)"#).unwrap();
        for caps in re_panneau.captures_iter(html) {
            let name = caps[1].trim().to_string();
            let path = caps[2].trim().to_string();

            // Ignore template or placeholder calls in documentation/comments
            if name.eq_ignore_ascii_case("nom") || path.eq_ignore_ascii_case("url") {
                continue;
            }

            let full_url = if path.starts_with("http") {
                path
            } else if path.starts_with('/') {
                format!("https://anime-sama.to{}", path)
            } else {
                format!("{}/{}/", clean_base, path.trim_matches('/'))
            };

            let clean_target = full_url.trim_end_matches('/').to_string() + "/";
            if !seen_urls.contains(&clean_target) {
                seen_urls.insert(clean_target.clone());
                seasons.push(SeasonItem {
                    name,
                    url: clean_target,
                });
            }
        }

        // Method 2: <select> dropdown for seasons (e.g. from a season page)
        // Real HTML: <label class="ct-select-saison"><select aria-label="Changer de saison">
        // No .as-dd-natif class on select in real page HTML
        if seasons.is_empty() {
            let document = Html::parse_document(html);
            // Broad selector: any option inside a season-label or any select with aria-label containing "saison"
            let selectors_to_try = [
                "label.ct-select-saison select option",
                "select[aria-label] option",
                "select.as-dd-natif option",
            ];
            for sel_str in &selectors_to_try {
                if !seasons.is_empty() { break; }
                let Ok(opt_sel) = Selector::parse(sel_str) else { continue };
                for opt in document.select(&opt_sel) {
                    let name = opt.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    if let Some(val) = opt.value().attr("value") {
                        let val_trimmed = val.trim();
                        if !val_trimmed.is_empty() && !val_trimmed.contains("/scan/") && (val_trimmed.contains("/saison") || val_trimmed.contains("/film") || val_trimmed.contains("/oav")) {
                            let full_url = if val_trimmed.starts_with("http") {
                                val_trimmed.to_string()
                            } else if val_trimmed.starts_with('/') {
                                format!("https://anime-sama.to{}", val_trimmed)
                            } else {
                                format!("{}/{}/", clean_base, val_trimmed.trim_matches('/'))
                            };
                            let clean_target = full_url.trim_end_matches('/').to_string() + "/";
                            if !seen_urls.contains(&clean_target) {
                                seen_urls.insert(clean_target.clone());
                                seasons.push(SeasonItem {
                                    name: if name.is_empty() { format!("Saison {}", seasons.len() + 1) } else { name },
                                    url: clean_target,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Method 3: <a class="oe-panneau"> links
        if seasons.is_empty() {
            let document = Html::parse_document(html);
            if let Ok(panneau_sel) = Selector::parse("a.oe-panneau") {
                for a in document.select(&panneau_sel) {
                    let name = a.text().collect::<Vec<_>>().join(" ").trim().to_string();
                    if let Some(href) = a.value().attr("href") {
                        if !href.contains("/scan/") && (href.contains("/saison") || href.contains("/film") || href.contains("/oav")) {
                            let full_url = if href.starts_with("http") {
                                href.to_string()
                            } else if href.starts_with('/') {
                                format!("https://anime-sama.to{}", href)
                            } else {
                                format!("{}/{}/", clean_base, href.trim_matches('/'))
                            };
                            let clean_target = full_url.trim_end_matches('/').to_string() + "/";
                            if !seen_urls.contains(&clean_target) {
                                seen_urls.insert(clean_target.clone());
                                seasons.push(SeasonItem {
                                    name: if name.is_empty() { format!("Saison {}", seasons.len() + 1) } else { name },
                                    url: clean_target,
                                });
                            }
                        }
                    }
                }
            }
        }

        seasons
    }

    pub async fn get_seasons(&self, anime_url: &str) -> Result<Vec<SeasonItem>, Box<dyn std::error::Error + Send + Sync>> {
        let clean_anime_url = anime_url.trim_end_matches('/');
        let is_vf = clean_anime_url.ends_with("/vf") || clean_anime_url.contains("/vf/");
        let catalogue_root = self.get_catalogue_root(anime_url);

        let mut seasons = Vec::new();

        // 1. Fetch catalogue page to find all seasons
        if let Ok(resp) = self.client.get(&format!("{}/", catalogue_root.trim_end_matches('/'))).send().await {
            if let Ok(html) = resp.text().await {
                seasons = self.parse_seasons_html(&html, &catalogue_root);
            }
        }

        // 2. If fetching catalogue root didn't yield seasons and anime_url differs, try anime_url
        if seasons.is_empty() && anime_url != catalogue_root {
            if let Ok(resp) = self.client.get(anime_url).send().await {
                if let Ok(html) = resp.text().await {
                    seasons = self.parse_seasons_html(&html, &catalogue_root);
                }
            }
        }

        // 3. Fallback: single Season 1
        if seasons.is_empty() {
            let season_url = if is_vf {
                format!("{}/saison1/vf/", catalogue_root.trim_end_matches('/'))
            } else {
                format!("{}/saison1/vostfr/", catalogue_root.trim_end_matches('/'))
            };
            return Ok(vec![SeasonItem {
                name: "Saison 1".to_string(),
                url: season_url,
            }]);
        }

        // 4. Adapt season URLs according to language (VF or VOSTFR)
        if is_vf {
            for s in &mut seasons {
                if s.url.contains("/vostfr") {
                    s.url = s.url.replace("/vostfr", "/vf");
                }
            }
        } else {
            for s in &mut seasons {
                if !s.url.contains("/scan/") && s.url.contains("/vf") {
                    s.url = s.url.replace("/vf", "/vostfr");
                }
            }
        }

        Ok(seasons)
    }

    pub async fn get_episodes(&self, anime_url: &str) -> Result<Vec<EpisodeItem>, Box<dyn std::error::Error + Send + Sync>> {
        let clean_url = anime_url.trim_end_matches('/');
        let is_vf = clean_url.ends_with("/vf") || clean_url.contains("/vf/");
        let target_url = if !clean_url.contains("/saison") && !clean_url.contains("/film") && !clean_url.contains("/oav") {
            let root = self.get_catalogue_root(clean_url);
            if is_vf {
                format!("{}/saison1/vf/", root.trim_end_matches('/'))
            } else {
                format!("{}/saison1/vostfr/", root.trim_end_matches('/'))
            }
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

    pub fn extract_episode_players_from_scripts(&self, text: &str, ep_idx: usize) -> Vec<String> {
        let mut players = Vec::new();
        let re_eps = regex::Regex::new(r#"var\s+eps\d*\s*=\s*\[(.*?)\]"#).unwrap();
        let re_url = regex::Regex::new(r#"["']((?:https?://|lien:)[^"']+)["']"#).unwrap();

        for caps in re_eps.captures_iter(text) {
            if let Some(array_content) = caps.get(1) {
                let urls: Vec<String> = re_url
                    .captures_iter(array_content.as_str())
                    .map(|c| c[1].to_string())
                    .collect();

                if ep_idx < urls.len() {
                    let ep_url = urls[ep_idx].trim().to_string();
                    if !ep_url.is_empty() && !players.contains(&ep_url) {
                        players.push(ep_url);
                    }
                }
            }
        }

        players
    }

    pub async fn get_player_iframe(&self, episode_url: &str, preferred_players: &[String]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let iframes = self.get_player_iframes(episode_url, preferred_players).await?;
        iframes.into_iter().next().ok_or_else(|| "No player iframe found".into())
    }

    pub async fn get_player_iframes(&self, episode_url: &str, preferred_players: &[String]) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let base_url = episode_url.split('#').next().unwrap_or(episode_url);
        let ep_num = self.extract_episode_number(episode_url).unwrap_or(1);
        let ep_idx = ep_num.saturating_sub(1);

        // Fetch page HTML to check for player scripts or iframes
        if let Ok(resp) = self.client.get(base_url).send().await {
            if let Ok(html) = resp.text().await {
                // Extract epsJeton if present (Anime-Sama token for /api/lien.php)
                let re_jeton = regex::Regex::new(r#"epsJeton\s*=\s*['"]([^'"]+)['"]"#).unwrap();
                let eps_jeton = re_jeton.captures(&html).and_then(|c| c.get(1)).map(|m| m.as_str().to_string());

                // Try extracting episode players from HTML scripts
                let mut candidate_urls = self.extract_episode_players_from_scripts(&html, ep_idx);

                // If no player arrays found in HTML, try fetching external episodes.js
                if candidate_urls.is_empty() {
                    let js_url = format!("{}/episodes.js", base_url.trim_end_matches('/'));
                    if let Ok(js_resp) = self.client.get(&js_url).send().await {
                        if let Ok(js_text) = js_resp.text().await {
                            candidate_urls = self.extract_episode_players_from_scripts(&js_text, ep_idx);
                        }
                    }
                }

                // Resolve any "lien:<id>" using Anime-Sama /api/lien.php
                for url in &mut candidate_urls {
                    if url.starts_with("lien:") {
                        let lien_id = url.trim_start_matches("lien:");
                        if let Some(ref jeton) = eps_jeton {
                            let api_url = format!("https://anime-sama.to/api/lien.php?j={}&l={}", urlencoding::encode(jeton), lien_id);
                            if let Ok(api_resp) = self.client.get(&api_url)
                                .header("Referer", "https://anime-sama.to/")
                                .header("X-Requested-With", "XMLHttpRequest")
                                .send().await
                            {
                                if let Ok(json) = api_resp.json::<serde_json::Value>().await {
                                    if let Some(real_url) = json.get("u").and_then(|v| v.as_str()) {
                                        *url = real_url.to_string();
                                    }
                                }
                            }
                        }
                    }
                }

                // Filter out unresolved "lien:" strings
                candidate_urls.retain(|u| !u.starts_with("lien:"));

                // If candidate URLs found for this episode, match against preferred_players priority
                if !candidate_urls.is_empty() {
                    let mut ordered = Vec::new();
                    for pref in preferred_players {
                        let pref_lower = pref.to_lowercase();
                        for u in &candidate_urls {
                            if u.to_lowercase().contains(&pref_lower) && !ordered.contains(u) {
                                ordered.push(u.clone());
                            }
                        }
                    }
                    for u in &candidate_urls {
                        if !ordered.contains(u) {
                            ordered.push(u.clone());
                        }
                    }
                    return Ok(ordered);
                }

                // Fallback: Look for embed links inside HTML scripts matching preferred players
                let mut fallback_urls = Vec::new();
                for pref in preferred_players {
                    let pref_lower = pref.to_lowercase();
                    let re_pattern = format!(r#"["'](https?://[^"']*{}[^"']*)["']"#, pref_lower);
                    if let Ok(re) = regex::Regex::new(&re_pattern) {
                        if let Some(caps) = re.captures(&html) {
                            if let Some(matched) = caps.get(1) {
                                let found = matched.as_str().to_string();
                                if !fallback_urls.contains(&found) {
                                    fallback_urls.push(found);
                                }
                            }
                        }
                    }
                }
                if !fallback_urls.is_empty() {
                    return Ok(fallback_urls);
                }

                // Fallback: Check iframe#playerDF
                let document = Html::parse_document(&html);
                if let Ok(iframe_sel) = Selector::parse("iframe#playerDF, iframe.playerDF, iframe") {
                    for iframe in document.select(&iframe_sel) {
                        if let Some(src) = iframe.value().attr("src") {
                            if !src.is_empty() && !src.contains("cadre") && !src.contains("adcash") && !src.contains("btag") {
                                return Ok(vec![src.to_string()]);
                            }
                        }
                    }
                }

                // General embed matcher
                let re_embed = regex::Regex::new(r#"["'](https?://(?:ansembed\.net|vidmoly\.[a-z]+|uqload\.[a-z]+|voe\.sx)[^"']*)["']"#).unwrap();
                let mut embeds = Vec::new();
                for caps in re_embed.captures_iter(&html) {
                    if let Some(matched) = caps.get(1) {
                        let u = matched.as_str().to_string();
                        if !embeds.contains(&u) {
                            embeds.push(u);
                        }
                    }
                }
                if !embeds.is_empty() {
                    return Ok(embeds);
                }
            }
        }

        // Fallback: return episode_url for yt-dlp to handle
        Ok(vec![episode_url.to_string()])
    }

    pub async fn resolve_video_url(&self, iframe_url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let clean_url = iframe_url.split('#').next().unwrap_or(iframe_url);
        
        // Sibnet is natively supported by yt-dlp's SibnetEmbed extractor
        if clean_url.contains("sibnet.ru") {
            return Ok(clean_url.to_string());
        }

        // Build request: Uqload blocks requests that send Referer: uqload.vc or anime-sama.to
        // with "Video embed restricted for this domain". Without Referer, it serves the full page.
        let mut req = self.client.get(clean_url);
        if clean_url.contains("vidmoly") || clean_url.contains("ansembed") {
            req = req.header("Referer", "https://anime-sama.to/");
        } else if !clean_url.contains("uqload") {
            req = req.header("Referer", "https://anime-sama.to/");
        }

        if let Ok(resp) = req.send().await {
            if let Ok(text) = resp.text().await {
                // If page returned embed restriction, do not treat as media
                if text.contains("restricted for this domain") {
                    return Err(format!("Player embed restricted for {}", clean_url).into());
                }

                // Try unpacking Dean Edwards JS (used by Uqload, Vidmoly, etc.)
                if let Some(unpacked) = unpack_dean_edwards(&text) {
                    if let Some(media_url) = extract_media_url_from_text(&unpacked) {
                        return Ok(media_url);
                    }
                }

                // Check direct media URL in HTML / script tags
                if let Some(media_url) = extract_media_url_from_text(&text) {
                    return Ok(media_url);
                }

                // Check HTML video source tag
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

        // If this was an embed player page and we couldn't resolve a video stream,
        // fail explicitly so the fallback system can try the next player.
        if clean_url.contains("uqload") || clean_url.contains("vidmoly") || clean_url.contains("ansembed") || clean_url.contains(".html") {
            return Err(format!("Could not resolve stream from player {}", clean_url).into());
        }

        Ok(clean_url.to_string())
    }
}

pub fn unpack_dean_edwards(packed: &str) -> Option<String> {
    let re = regex::Regex::new(
        r#"eval\(function\(p,a,c,k,e,[rd]\)\{.*?\}\(\s*['"](?P<p>.+?)['"]\s*,\s*(?P<a>\d+)\s*,\s*(?P<c>\d+)\s*,\s*['"](?P<k>.+?)['"]\.split\(['"]\|['"]\)\s*\)"#,
    ).ok()?;

    let caps = re.captures(packed)?;
    let p = caps.name("p")?.as_str();
    let a: u32 = caps.name("a")?.as_str().parse().ok()?;
    let c: usize = caps.name("c")?.as_str().parse().ok()?;
    let k_str = caps.name("k")?.as_str();
    let k: Vec<&str> = k_str.split('|').collect();

    fn to_base(mut n: usize, radix: u32) -> String {
        if n == 0 {
            return "0".to_string();
        }
        let digits = b"0123456789abcdefghijklmnopqrstuvwxyz";
        let mut res = Vec::new();
        let r = radix as usize;
        while n > 0 {
            res.push(digits[n % r]);
            n /= r;
        }
        res.reverse();
        String::from_utf8(res).unwrap_or_default()
    }

    let mut dict: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for i in 0..c {
        let key = to_base(i, a);
        let val = if i < k.len() && !k[i].is_empty() {
            k[i].to_string()
        } else {
            key.clone()
        };
        dict.insert(key, val);
    }

    let word_re = regex::Regex::new(r"\b\w+\b").ok()?;
    let unpacked = word_re.replace_all(p, |word_caps: &regex::Captures| {
        let word = word_caps.get(0).map_or("", |m| m.as_str());
        if let Some(val) = dict.get(word) {
            val.clone()
        } else {
            word.to_string()
        }
    });

    Some(unpacked.into_owned())
}

pub fn extract_media_url_from_text(text: &str) -> Option<String> {
    // 1. Check sources: [{ file: "..." }] or sources:[{file:"..."}]
    if let Ok(re_sources) = regex::Regex::new(r#"sources\s*:\s*\[\s*\{\s*file\s*:\s*["']([^"']+)["']"#) {
        if let Some(caps) = re_sources.captures(text) {
            if let Some(url) = caps.get(1) {
                return Some(url.as_str().to_string());
            }
        }
    }

    // 2. Check file: "..." with m3u8 or mp4
    if let Ok(re_file) = regex::Regex::new(r#"file\s*:\s*["'](https?://[^"']+\.(?:m3u8|mp4)[^"']*)["']"#) {
        if let Some(caps) = re_file.captures(text) {
            if let Some(url) = caps.get(1) {
                return Some(url.as_str().to_string());
            }
        }
    }

    // 3. Check any direct https?://...mp4 or https?://...m3u8
    if let Ok(re_any) = regex::Regex::new(r#"https?://[^\s"'<>]+\.(?:m3u8|mp4)[^\s"'<>]*"#) {
        if let Some(mat) = re_any.find(text) {
            return Some(mat.as_str().to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_episode_players_from_scripts() {
        let scraper = AnimeSama::new();
        let js = r#"
            var eps1 = ['https://vidmoly.to/embed-ep1.html', 'https://vidmoly.to/embed-ep2.html'];
            var eps2 = ['https://voe.sx/e/ep1', 'https://voe.sx/e/ep2'];
            var eps3 = ['https://sibnet.ru/ep1', 'https://sibnet.ru/ep2'];
        "#;

        // Episode 1 (idx 0)
        let ep1_players = scraper.extract_episode_players_from_scripts(js, 0);
        assert_eq!(ep1_players.len(), 3);
        assert_eq!(ep1_players[0], "https://vidmoly.to/embed-ep1.html");
        assert_eq!(ep1_players[1], "https://voe.sx/e/ep1");
        assert_eq!(ep1_players[2], "https://sibnet.ru/ep1");

        // Episode 2 (idx 1)
        let ep2_players = scraper.extract_episode_players_from_scripts(js, 1);
        assert_eq!(ep2_players.len(), 3);
        assert_eq!(ep2_players[0], "https://vidmoly.to/embed-ep2.html");
        assert_eq!(ep2_players[1], "https://voe.sx/e/ep2");
        assert_eq!(ep2_players[2], "https://sibnet.ru/ep2");

        // Episode 3 (idx 2, does not exist)
        let ep3_players = scraper.extract_episode_players_from_scripts(js, 2);
        assert!(ep3_players.is_empty());
    }

    #[test]
    fn test_preferred_player_resolution() {
        let scraper = AnimeSama::new();
        let js = r#"
            var eps1 = ['https://vidmoly.to/embed-ep1.html'];
            var eps2 = ['https://voe.sx/e/ep1'];
            var eps3 = ['https://sibnet.ru/ep1'];
        "#;

        let players = scraper.extract_episode_players_from_scripts(js, 0);

        // When VOE is preferred over Vidmoly
        let prefs = vec!["VOE".to_string(), "Vidmoly".to_string()];
        let matched = prefs
            .iter()
            .find_map(|p| players.iter().find(|u| u.to_lowercase().contains(&p.to_lowercase())))
            .unwrap();
        assert_eq!(matched, "https://voe.sx/e/ep1");

        // When Vidmoly is preferred over VOE
        let prefs2 = vec!["Vidmoly".to_string(), "VOE".to_string()];
        let matched2 = prefs2
            .iter()
            .find_map(|p| players.iter().find(|u| u.to_lowercase().contains(&p.to_lowercase())))
            .unwrap();
        assert_eq!(matched2, "https://vidmoly.to/embed-ep1.html");
    }

    #[test]
    fn test_get_catalogue_root() {
        let scraper = AnimeSama::new();
        assert_eq!(
            scraper.get_catalogue_root("https://anime-sama.to/catalogue/shingeki-no-kyojin/"),
            "https://anime-sama.to/catalogue/shingeki-no-kyojin"
        );
        assert_eq!(
            scraper.get_catalogue_root("https://anime-sama.to/catalogue/shingeki-no-kyojin/saison1/vostfr/"),
            "https://anime-sama.to/catalogue/shingeki-no-kyojin"
        );
    }

    #[test]
    fn test_parse_seasons_html_panneau_anime() {
        let scraper = AnimeSama::new();
        let html = r#"
            <script>
            /*panneauAnime("nom", "url"); -> placeholder comment */
            panneauAnime("Saison 1", "saison1/vostfr");
            panneauAnime("Film", "film/vostfr");
            panneauAnime("Saison 2", "saison2/vostfr");
            panneauAnime("Saison 3", "saison3/vostfr");
            </script>
        "#;
        let seasons = scraper.parse_seasons_html(html, "https://anime-sama.to/catalogue/jujutsu-kaisen");
        assert_eq!(seasons.len(), 4);
        assert_eq!(seasons[0].name, "Saison 1");
        assert_eq!(seasons[0].url, "https://anime-sama.to/catalogue/jujutsu-kaisen/saison1/vostfr/");
        assert_eq!(seasons[1].name, "Film");
        assert_eq!(seasons[1].url, "https://anime-sama.to/catalogue/jujutsu-kaisen/film/vostfr/");
        assert_eq!(seasons[2].name, "Saison 2");
        assert_eq!(seasons[2].url, "https://anime-sama.to/catalogue/jujutsu-kaisen/saison2/vostfr/");
        assert_eq!(seasons[3].name, "Saison 3");
        assert_eq!(seasons[3].url, "https://anime-sama.to/catalogue/jujutsu-kaisen/saison3/vostfr/");
    }

    #[test]
    fn test_parse_seasons_html_select_dropdown() {
        let scraper = AnimeSama::new();
        let html = r#"
            <label class="ct-select-saison">
                <select class="as-dd-natif" aria-label="Changer de saison">
                    <option value="/catalogue/snk/saison1/vostfr/">Saison 1</option>
                    <option value="/catalogue/snk/saison2/vostfr/">Saison 2</option>
                    <option value="/catalogue/snk/film/vostfr/">Film</option>
                    <option value="/catalogue/snk/scan/vf/">Scans</option>
                </select>
            </label>
        "#;
        let seasons = scraper.parse_seasons_html(html, "https://anime-sama.to/catalogue/snk");
        assert_eq!(seasons.len(), 3);
        assert_eq!(seasons[0].name, "Saison 1");
        assert_eq!(seasons[0].url, "https://anime-sama.to/catalogue/snk/saison1/vostfr/");
        assert_eq!(seasons[1].name, "Saison 2");
        assert_eq!(seasons[1].url, "https://anime-sama.to/catalogue/snk/saison2/vostfr/");
        assert_eq!(seasons[2].name, "Film");
        assert_eq!(seasons[2].url, "https://anime-sama.to/catalogue/snk/film/vostfr/");
    }

    #[test]
    fn test_parse_seasons_html_real_season_page() {
        // Real HTML from /saison1/vostfr/ page: select has NO .as-dd-natif class
        let scraper = AnimeSama::new();
        let html = r#"
            <label class="ct-select-saison">
                <span class="ct-etiquette">Saison</span>
                <select onchange="if (this.value) location.href = this.value;" aria-label="Changer de saison">
                    <option value="/catalogue/shingeki-no-kyojin/saison1/vostfr/" selected="">Saison 1</option>
                    <option value="/catalogue/shingeki-no-kyojin/saison2/vostfr/">Saison 2</option>
                    <option value="/catalogue/shingeki-no-kyojin/saison3/vostfr/">Saison 3</option>
                    <option value="/catalogue/shingeki-no-kyojin/saison4/vostfr/">Saison 4</option>
                    <option value="/catalogue/shingeki-no-kyojin/film/vostfr/">Film</option>
                    <option value="/catalogue/shingeki-no-kyojin/oav/vostfr/">OAV</option>
                    <option value="/catalogue/shingeki-no-kyojin/scan/vf/">Scans</option>
                </select>
            </label>
        "#;
        let seasons = scraper.parse_seasons_html(html, "https://anime-sama.to/catalogue/shingeki-no-kyojin");
        assert_eq!(seasons.len(), 6, "Expected 6 seasons (excluding Scans), got: {:?}", seasons);
        assert_eq!(seasons[0].name, "Saison 1");
        assert_eq!(seasons[0].url, "https://anime-sama.to/catalogue/shingeki-no-kyojin/saison1/vostfr/");
        assert_eq!(seasons[3].name, "Saison 4");
        assert_eq!(seasons[4].name, "Film");
        assert_eq!(seasons[5].name, "OAV");
    }

    #[test]
    fn test_unpack_dean_edwards_and_extract_url() {
        let packed = r#"eval(function(p,a,c,k,e,d){while(c--)if(k[c])p=p.replace(new RegExp('\\b'+c.toString(a)+'\\b','g'),k[c]);return p}('r("4c").bh({bg:[{23:"1j://2u.1b.1a/bf/2t/2s/be,l,n,h,.bd/bc.bb?t=ba&s=4v&e=b9&v=4w&i=0.0&b8=0"}],b7:"1j://2u.1b.1a/i/2t/2s/2h.b6"});',36,414,'||||||||||jw|function||||player|if|||tracks|var||||settings|submenu|svg|jwplayer|||on||||audioTracks|||||||589|lastt||name||vc|uqload|icon|div|position|event|false|aria|attr|https|length|active|true|path|100|captions|addButton|log|console|hide|tt|sec|769|240|60009|dd|adb||date|file|||current_audio||return|default_audio|localStorage|currentTrack||removeClass|expanded|checked|open|f2zixqxw8dwe|rewind|seek|height|width|tr|ttf2zixqxw8dwe|ls|vvplay|vvad|kbps|06070|03|strm14||setCurrentAudioTrack|audio_name|for|getAudioTracks||audio_set||getItem|addClass|controls|playbackRates|images|win|window|detach|ff00|button|getPosition|974|887|013|867|178|focusable|viewBox||class|2000|org|w3|www|http|xmlns|ff11|06475|23525|29374|97928|30317|31579|29683||38421|||30626|||||72072|H|vplayer|appendTo|play|remove|no|url|top|margin|zIndex|css|openIframeOverlay|op|pause|empty|RegExp|track_name|get|data|view|1794676241|4530506|video_ad|doPlay|value|cookie||document|new|prevt|logo|kind||srt|vtt|300|setTimeout|setItem|audioTrackChanged|dualSound|else|child|last|menu|quality|hasClass|toggleClass|Track|Audio|dualy|mousedown|buttons|topbar|download11|focus|_blank|html|site|Watch|png|download2||playback||insertBefore|Rewind|778Z|214|2A4|3H209|3v19|9c4|7l41|9a6|3c0|1v19|4H79|3h48|8H146|3a4|2v125|130|1Zm162|4v62|13a4|51l|278Zm|278|1S103|1s6|3Zm|078a21||131|M113|insertAfter|Forward|69999|88605|21053||03598|02543|99999|72863|77056|04577|422413|163|210431|860275|03972|689569|893957|124979|52502|174985|57502||04363|13843|480087|93574|99396|160|76396|164107|63589||03604|125|778|993957|rewind2|this|click|scrolling|frameborder|src|prop|50px|1000001|iframe|center|align|text|rgba|background|1000000|left|absolute|file_code|upload_srt|setCurrentCaptions|id|track|test|captionsChanged|ready|set_audio_track|uqResumeAsk|a9b56dbc8f3e7f2966f5745561a775b8|hash|view_id|dl|adbon|over_player_msg|Secure|None|SameSite|domain|toGMTString|expires|1000|||getTime|setTime|Date|createCookieSec|match|readCookie|const|bufferChange|show|complete|ttl|round|Math|set|slow|fadeIn|video_ad_fadein|time|cache|Cache|Content|headers|ajaxSetup|v2done|tott|vastdone2|vastdone1|playbackRateControls|cast|bar|control|link|images2|premium|aboutlink|UQLOAD|abouttext|netflix2|skin|545|360p|544|1469|720p|1468|4016|1080p|4015|qualityLabels|fontOpacity|backgroundOpacity|Tahoma|fontFamily|303030|backgroundColor|FFFFFF|color|userFontScale||Upload|label|thumbnails|f2zixqxw8dwe_sli|metadata|preload|landscape|fullscreenOrientationLock|1535|duration|uniform|stretching|jpg|image|sp|14400|ViC_lDBS9_yitW8pgbXA9zGRnzAfAV4W7KZb4Imw8mQ|m3u8|master|urlset|f2zixqxw8dwe_|hls2|sources|setup'.split('|'))"#;

        let unpacked = unpack_dean_edwards(packed);
        assert!(unpacked.is_some());
        let unpacked_str = unpacked.unwrap();
        assert!(unpacked_str.contains("jwplayer(\"vplayer\")"));

        let media_url = extract_media_url_from_text(&unpacked_str);
        assert!(media_url.is_some());
        let url = media_url.unwrap();
        assert!(url.contains("strm14.uqload.vc"));
        assert!(url.contains("master.m3u8"));
    }

    #[test]
    fn test_extract_vidmoly_source() {
        let html = r#"
            <script>
                var player = jwplayer("vplayer");
                player.setup({
                    sources: [{ file: 'https://prx-1356-ant-w.vmpx.online/hls2/01/02971/xfnnx1ovu1bb_,n,l,.urlset/master.m3u8?t=test123' }],
                    image: 'https://imgvm.com/i1745/i/01/02971/xfnnx1ovu1bb.jpg'
                });
            </script>
        "#;
        let media_url = extract_media_url_from_text(html);
        assert!(media_url.is_some());
        assert_eq!(
            media_url.unwrap(),
            "https://prx-1356-ant-w.vmpx.online/hls2/01/02971/xfnnx1ovu1bb_,n,l,.urlset/master.m3u8?t=test123"
        );
    }
}
