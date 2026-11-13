use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use tokio::process::Command;
use tokio::sync::mpsc::Sender;

use crate::app::DownloadProgress;

pub struct Downloader {
    client: Client,
    pub output_dir: String,
    max_retries: usize,
}

impl Downloader {
    pub fn new(output_dir: String) -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_default();
        Self {
            client,
            output_dir,
            max_retries: 3,
        }
    }

    pub fn is_embed_player(url: &str) -> bool {
        let lower = url.to_lowercase();
        lower.contains("streamtape")
            || lower.contains("voe")
            || lower.contains("mfw09")
            || lower.contains("voembed")
            || lower.contains("get_video")
            || lower.contains("ansembed")
            || lower.contains("vidmoly")
            || lower.contains("uqload")
            || lower.contains("earnvid")
            || lower.contains("sibnet")
            || lower.contains("sendvid")
            || lower.contains("dood")
            || lower.contains("mytv")
            || lower.contains("moon")
    }

    /// Universal download method using yt-dlp which natively supports all embed players
    pub async fn download_with_ytdlp(
        &self,
        target_url: &str,
        ep_num: usize,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        fs::create_dir_all(&self.output_dir)?;
        let output_template = format!("{}/episode_{:02}.%(ext)s", self.output_dir, ep_num);
        let final_filename = format!("{}/episode_{:02}.mp4", self.output_dir, ep_num);

        let status = Command::new("yt-dlp")
            .arg("-o")
            .arg(&output_template)
            .arg("--no-playlist")
            .arg(target_url)
            .status()
            .await;

        match status {
            Ok(s) if s.success() => Ok(final_filename),
            _ => Err("yt-dlp download failed or yt-dlp is not installed on the system.".into()),
        }
    }

    /// Asynchronous download with MPSC progress channel for TUI
    pub async fn download_episode_with_progress(
        &self,
        url: &str,
        ep_num: usize,
        tx: Option<Sender<DownloadProgress>>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        fs::create_dir_all(&self.output_dir)?;
        let filename = format!("{}/episode_{:02}.mp4", self.output_dir, ep_num);

        if let Some(ref sender) = tx {
            let _ = sender.send(DownloadProgress {
                ep_number: ep_num,
                title: format!("Episode {:02}", ep_num),
                downloaded_bytes: 0,
                total_bytes: 0,
                percent: 0.0,
                status: "Starting download...".to_string(),
                is_finished: false,
                error: None,
            }).await;
        }

        // Try yt-dlp first if it's an embed player
        if Self::is_embed_player(url) {
            if let Some(ref sender) = tx {
                let _ = sender.send(DownloadProgress {
                    ep_number: ep_num,
                    title: format!("Episode {:02}", ep_num),
                    downloaded_bytes: 0,
                    total_bytes: 0,
                    percent: 0.0,
                    status: "Downloading via yt-dlp backend...".to_string(),
                    is_finished: false,
                    error: None,
                }).await;
            }

            match self.download_with_ytdlp(url, ep_num).await {
                Ok(path) => {
                    if let Some(ref sender) = tx {
                        let _ = sender.send(DownloadProgress {
                            ep_number: ep_num,
                            title: format!("Episode {:02}", ep_num),
                            downloaded_bytes: 0,
                            total_bytes: 0,
                            percent: 100.0,
                            status: "Download completed successfully (yt-dlp)!".to_string(),
                            is_finished: true,
                            error: None,
                        }).await;
                    }
                    return Ok(path);
                }
                Err(e) => {
                    if let Some(ref sender) = tx {
                        let _ = sender.send(DownloadProgress {
                            ep_number: ep_num,
                            title: format!("Episode {:02}", ep_num),
                            downloaded_bytes: 0,
                            total_bytes: 0,
                            percent: 0.0,
                            status: format!("yt-dlp fallback to HTTP: {}", e),
                            is_finished: false,
                            error: None,
                        }).await;
                    }
                }
            }
        }

        // HTTP download with retries
        for attempt in 1..=self.max_retries {
            match self.try_download_with_tx(url, &filename, ep_num, tx.as_ref()).await {
                Ok(_) => {
                    if let Some(ref sender) = tx {
                        let _ = sender.send(DownloadProgress {
                            ep_number: ep_num,
                            title: format!("Episode {:02}", ep_num),
                            downloaded_bytes: 0,
                            total_bytes: 0,
                            percent: 100.0,
                            status: "Download completed successfully!".to_string(),
                            is_finished: true,
                            error: None,
                        }).await;
                    }
                    return Ok(filename);
                }
                Err(e) => {
                    if attempt == self.max_retries {
                        if let Some(ref sender) = tx {
                            let _ = sender.send(DownloadProgress {
                                ep_number: ep_num,
                                title: format!("Episode {:02}", ep_num),
                                downloaded_bytes: 0,
                                total_bytes: 0,
                                percent: 0.0,
                                status: format!("Download failed: {}", e),
                                is_finished: true,
                                error: Some(e.to_string()),
                            }).await;
                        }
                        return Err(e);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        }

        Err("Download failed after all attempts".into())
    }

    async fn try_download_with_tx(
        &self,
        url: &str,
        filename: &str,
        ep_num: usize,
        tx: Option<&Sender<DownloadProgress>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let resp = self.client.head(url).send().await?;
        let total_size = resp
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|val| val.to_str().ok())
            .and_then(|val| val.parse::<u64>().ok())
            .unwrap_or(0);

        let path = Path::new(filename);
        let mut downloaded = 0;
        let mut file_mode = "wb";

        if path.exists() {
            let metadata = fs::metadata(path)?;
            downloaded = metadata.len();
            if total_size > 0 && downloaded == total_size {
                return Ok(());
            } else if downloaded > total_size && total_size > 0 {
                downloaded = 0;
            } else if downloaded > 0 {
                file_mode = "ab";
            }
        }

        let mut req = self.client.get(url);
        if downloaded > 0 {
            req = req.header(reqwest::header::RANGE, format!("bytes={}-", downloaded));
        }

        let mut res = req.send().await?;
        let mut file = if file_mode == "ab" {
            OpenOptions::new().append(true).open(path)?
        } else {
            fs::File::create(path)?
        };

        let mut last_report = std::time::Instant::now();
        while let Some(chunk) = res.chunk().await? {
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;

            if last_report.elapsed() >= std::time::Duration::from_millis(200) {
                last_report = std::time::Instant::now();
                if let Some(sender) = tx {
                    let percent = if total_size > 0 {
                        (downloaded as f64 / total_size as f64) * 100.0
                    } else {
                        0.0
                    };
                    let _ = sender.send(DownloadProgress {
                        ep_number: ep_num,
                        title: format!("Episode {:02}", ep_num),
                        downloaded_bytes: downloaded,
                        total_bytes: total_size,
                        percent,
                        status: format!("{:.1} MB / {:.1} MB ({:.1}%)",
                            downloaded as f64 / 1_048_576.0,
                            total_size as f64 / 1_048_576.0,
                            percent
                        ),
                        is_finished: false,
                        error: None,
                    }).await;
                }
            }
        }

        Ok(())
    }

    /// CLI progress-bar download method
    pub async fn download_episode(
        &self,
        url: &str,
        ep_num: usize,
        pb: ProgressBar,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        fs::create_dir_all(&self.output_dir)?;
        let filename = format!("{}/episode_{:02}.mp4", self.output_dir, ep_num);

        if Self::is_embed_player(url) {
            pb.set_message("Downloading via yt-dlp...");
            match self.download_with_ytdlp(url, ep_num).await {
                Ok(path) => {
                    pb.finish_with_message(format!("Episode {} downloaded successfully with yt-dlp", ep_num));
                    return Ok(path);
                }
                Err(_) => {
                    pb.set_message("yt-dlp failed, falling back to smart HTTP downloader...");
                }
            }
        }

        for attempt in 1..=self.max_retries {
            match self.try_download(url, &filename, &pb).await {
                Ok(_) => {
                    pb.finish_with_message(format!("Episode {} downloaded successfully", ep_num));
                    return Ok(filename);
                }
                Err(e) => {
                    if attempt == self.max_retries {
                        pb.finish_with_message(format!("Episode {} failed after {} attempts", ep_num, self.max_retries));
                        return Err(e);
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                }
            }
        }

        Err("Download failed".into())
    }

    async fn try_download(&self, url: &str, filename: &str, pb: &ProgressBar) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let resp = self.client.head(url).send().await?;
        let total_size = resp
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|val| val.to_str().ok())
            .and_then(|val| val.parse::<u64>().ok())
            .unwrap_or(0);

        let path = Path::new(filename);
        let mut downloaded = 0;
        let mut file_mode = "wb";

        if path.exists() {
            let metadata = fs::metadata(path)?;
            downloaded = metadata.len();
            if total_size > 0 && downloaded == total_size {
                pb.set_length(total_size);
                pb.set_position(total_size);
                return Ok(());
            } else if downloaded > total_size && total_size > 0 {
                downloaded = 0;
            } else if downloaded > 0 {
                file_mode = "ab";
            }
        }

        pb.set_length(total_size);
        pb.set_position(downloaded);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
                .progress_chars("#>-"),
        );

        let mut req = self.client.get(url);
        if downloaded > 0 {
            req = req.header(reqwest::header::RANGE, format!("bytes={}-", downloaded));
        }

        let mut res = req.send().await?;
        let mut file = if file_mode == "ab" {
            OpenOptions::new().append(true).open(path)?
        } else {
            fs::File::create(path)?
        };

        while let Some(chunk) = res.chunk().await? {
            file.write_all(&chunk)?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        Ok(())
    }
}
