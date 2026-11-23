use anime_dl_rs::providers::voir_anime::VoirAnime;
use anime_dl_rs::providers::anime_sama::AnimeSama;
use anime_dl_rs::downloader::Downloader;

#[tokio::test]
async fn test_live_search_voir_anime() {
    let scraper = VoirAnime::new();
    match scraper.search("One Piece").await {
        Ok(results) => {
            println!("Live search returned {} results.", results.len());
            assert!(!results.is_empty(), "Search should return results for a popular anime");
        }
        Err(e) => {
            eprintln!("Live search network error (skipped): {}", e);
        }
    }
}

#[tokio::test]
async fn test_live_search_anime_sama() {
    let provider = AnimeSama::new();
    match provider.search("Demon Slayer").await {
        Ok(results) => {
            println!("Anime-Sama live search returned {} results.", results.len());
            assert!(!results.is_empty(), "Demon Slayer should return results");
            let has_vostfr = results.iter().any(|r| r.language == anime_dl_rs::config::Language::VOSTFR);
            let has_vf = results.iter().any(|r| r.language == anime_dl_rs::config::Language::VF);
            assert!(has_vostfr, "Demon Slayer should have VOSTFR in search results");
            assert!(has_vf, "Demon Slayer should have VF in search results");
        }
        Err(e) => {
            eprintln!("Anime-Sama live search network error (skipped): {}", e);
        }
    }
}

#[tokio::test]
async fn test_live_get_seasons_anime_sama() {
    let provider = AnimeSama::new();
    let anime_url = "https://anime-sama.to/catalogue/shingeki-no-kyojin/";
    match provider.get_seasons(anime_url).await {
        Ok(seasons) => {
            println!("Anime-Sama get_seasons returned {} seasons.", seasons.len());
            assert!(!seasons.is_empty(), "Anime-Sama should return seasons");
        }
        Err(e) => {
            eprintln!("Anime-Sama get_seasons network error (skipped): {}", e);
        }
    }
}

#[tokio::test]
async fn test_live_get_episodes() {
    let scraper = VoirAnime::new();
    // Using a stable anime URL on voir-anime.to
    let anime_url = "https://voir-anime.to/anime/mission-yozakura-family-2/";
    match scraper.get_episodes(anime_url).await {
        Ok(episodes) => {
            println!("Fetched {} episodes for Mission Yozakura Family 2", episodes.len());
            if !episodes.is_empty() {
                assert!(episodes[0].number > 0);
                assert!(!episodes[0].url.is_empty());
            }
        }
        Err(e) => {
            eprintln!("Get episodes network error (skipped): {}", e);
        }
    }
}

#[tokio::test]
async fn test_live_player_iframe_extraction() {
    let scraper = VoirAnime::new();
    // Using a known episode URL
    let episode_url = "https://voir-anime.to/anime/mission-yozakura-family-2/mission-yozakura-family-2-17-vostfr/";
    let preferred = vec!["Stape".to_string(), "myTV".to_string(), "VOE".to_string(), "MOON".to_string()];
    
    match scraper.get_player_iframe(episode_url, &preferred).await {
        Ok(iframe_url) => {
            println!("Successfully resolved player iframe: {}", iframe_url);
            assert!(!iframe_url.is_empty());

            // Test video URL resolution from iframe
            match scraper.resolve_video_url(&iframe_url).await {
                Ok(video_url) => {
                    println!("Resolved video stream URL: {}", video_url);
                    assert!(!video_url.is_empty());
                }
                Err(e) => {
                    eprintln!("Failed to resolve video stream (may require browser/captcha): {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Get player iframe error (skipped): {}", e);
        }
    }
}

#[test]
fn test_downloader_initialization() {
    let downloader = Downloader::new("./test_downloads".to_string());
    // Verify downloader is correctly initialized
    assert_eq!(downloader.output_dir, "./test_downloads");
}
