use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use tokio::io::AsyncBufReadExt;
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

    /// Universal download method using yt-dlp which natively supports all embed players.
    /// When `tx` is Some, streams progress in real-time by parsing yt-dlp output line by line.
    /// When `tx` is None (CLI path), uses the original blocking `cmd.output()` behavior.
    pub async fn download_with_ytdlp(
        &self,
        target_url: &str,
        ep_num: usize,
        tx: Option<Sender<DownloadProgress>>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        fs::create_dir_all(&self.output_dir)?;
        let output_template = format!("{}/episode_{:02}.%(ext)s", self.output_dir, ep_num);
        let final_filename = format!("{}/episode_{:02}.mp4", self.output_dir, ep_num);

        let mut cmd = Command::new("yt-dlp");
        cmd.arg("-o")
            .arg(&output_template)
            .arg("--no-playlist")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        // When streaming progress, add --newline so each update is a separate line,
        // and --progress-template with a parseable format prefixed with "YTPROG|||"
        if tx.is_some() {
            cmd.arg("--newline")
                .arg("--progress-template")
                .arg("download:YTPROG|||%(progress.downloaded_bytes)s|||%(progress.total_bytes_estimate)s|||%(progress._percent_str)s|||%(progress._speed_str)s|||%(progress._eta_str)s");
        }

        if target_url.contains("voir-anime") || target_url.contains("voiranime") {
            cmd.arg("--referer").arg("https://voiranime.cx/");
        } else if target_url.contains("anime-sama") || target_url.contains("ansembed") {
            cmd.arg("--referer").arg("https://anime-sama.to/");
        } else if target_url.contains("/embed") && target_url.contains("uqload") {
            // Do not pass referer to uqload embed page, triggers restriction
        } else if target_url.contains("uqload") || target_url.contains("strm") {
            cmd.arg("--referer").arg("https://uqload.vc/");
        } else if target_url.contains("sibnet") {
            cmd.arg("--referer").arg("https://video.sibnet.ru/");
        } else if target_url.contains("vidmoly") {
            cmd.arg("--referer").arg("https://vidmoly.to/");
        } else if target_url.contains("sendvid") {
            cmd.arg("--referer").arg("https://sendvid.com/");
        } else if target_url.contains("voe") {
            cmd.arg("--referer").arg("https://voe.sx/");
        }

        cmd.arg(target_url);

        if let Some(ref sender) = tx {
            // === Streaming mode: spawn child, read stdout for progress and stderr for errors ===
            let mut child = cmd.spawn()
                .map_err(|e| format!("yt-dlp execution failed: {}", e))?;

            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            // Capture stderr in background to collect errors if process fails
            let stderr_task = tokio::spawn(async move {
                let mut err_lines = Vec::new();
                if let Some(stderr) = stderr {
                    let mut reader = tokio::io::BufReader::new(stderr);
                    let mut line = String::new();
                    while let Ok(n) = AsyncBufReadExt::read_line(&mut reader, &mut line).await {
                        if n == 0 {
                            break;
                        }
                        let trimmed = line.trim();
                        if trimmed.contains("ERROR:") || trimmed.contains("WARNING:") {
                            err_lines.push(trimmed.to_string());
                        }
                        line.clear();
                    }
                }
                err_lines
            });

            // Read stdout line by line: yt-dlp writes --progress-template output to stdout
            if let Some(stdout) = stdout {
                let mut reader = tokio::io::BufReader::new(stdout);
                let mut line = String::new();
                while let Ok(n) = AsyncBufReadExt::read_line(&mut reader, &mut line).await {
                    if n == 0 {
                        break;
                    }
                    let trimmed = line.trim();
                    if let Some(progress) = Self::parse_ytdlp_progress(trimmed, ep_num) {
                        let _ = sender.send(progress).await;
                    }
                    line.clear();
                }
            }

            let error_lines = stderr_task.await.unwrap_or_default();
            let status = child.wait().await
                .map_err(|e| format!("yt-dlp execution failed: {}", e))?;

            if !status.success() {
                let first_err = error_lines.into_iter()
                    .find(|l| l.contains("ERROR:"))
                    .unwrap_or_else(|| "yt-dlp download failed".to_string());
                return Err(first_err.into());
            }
        } else {
            // === Non-streaming mode: original blocking behavior (CLI path) ===
            let output = cmd.output().await;
            match output {
                Ok(out) if out.status.success() => {}
                Ok(out) => {
                    let err_msg = String::from_utf8_lossy(&out.stderr);
                    let first_err = err_msg
                        .lines()
                        .find(|l| l.contains("ERROR:"))
                        .map(|l| l.trim().to_string())
                        .unwrap_or_else(|| "yt-dlp download failed".to_string());
                    return Err(first_err.into());
                }
                Err(e) => return Err(format!("yt-dlp execution failed: {}", e).into()),
            }
        }

        // Post-download validation (shared by both paths)
        let html_file = format!("{}/episode_{:02}.html", self.output_dir, ep_num);
        let htm_file = format!("{}/episode_{:02}.htm", self.output_dir, ep_num);
        if Path::new(&html_file).exists() {
            let _ = fs::remove_file(&html_file);
            return Err("yt-dlp downloaded an HTML webpage instead of video media".into());
        }
        if Path::new(&htm_file).exists() {
            let _ = fs::remove_file(&htm_file);
            return Err("yt-dlp downloaded an HTML webpage instead of video media".into());
        }

        let possible_exts = ["mp4", "mkv", "webm", "ts"];
        for ext in &possible_exts {
            let candidate = format!("{}/episode_{:02}.{}", self.output_dir, ep_num, ext);
            if let Ok(meta) = fs::metadata(&candidate) {
                if meta.len() > 10_000 {
                    return Ok(candidate);
                }
            }
        }
        Ok(final_filename)
    }

    /// Parse a YTPROG-prefixed progress line from yt-dlp's --progress-template output.
    /// Format: "YTPROG|||<downloaded_bytes>|||<total_bytes>|||<percent>|||<speed>|||<eta>"
    fn parse_ytdlp_progress(line: &str, ep_num: usize) -> Option<DownloadProgress> {
        if !line.starts_with("YTPROG|||") {
            return None;
        }

        let parts: Vec<&str> = line.split("|||").collect();
        if parts.len() < 6 {
            return None;
        }

        let downloaded_bytes = parts[1].trim().parse::<u64>().unwrap_or(0);
        let total_bytes = parts[2].trim().parse::<u64>().unwrap_or(0);

        let mut percent = parts[3]
            .trim()
            .trim_end_matches('%')
            .trim()
            .parse::<f64>()
            .unwrap_or(0.0);

        if percent == 0.0 && total_bytes > 0 && downloaded_bytes > 0 {
            percent = (downloaded_bytes as f64 / total_bytes as f64) * 100.0;
        }

        let speed = parts[4].trim();
        let eta = parts[5].trim();

        let status = if total_bytes > 0 {
            format!(
                "{:.1} MB / {:.1} MB — {} ETA {}",
                downloaded_bytes as f64 / 1_048_576.0,
                total_bytes as f64 / 1_048_576.0,
                speed,
                eta
            )
        } else {
            format!(
                "{:.1} MB — {} ETA {}",
                downloaded_bytes as f64 / 1_048_576.0,
                speed,
                eta
            )
        };

        Some(DownloadProgress {
            ep_number: ep_num,
            title: format!("Episode {:02}", ep_num),
            downloaded_bytes,
            total_bytes,
            percent,
            status,
            is_finished: false,
            error: None,
        })
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

        // Try yt-dlp first if it's an embed player or an m3u8 playlist
        if Self::is_embed_player(url) || url.contains(".m3u8") {
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

            match self.download_with_ytdlp(url, ep_num, tx.clone()).await {
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
                    let is_webpage = url.contains(".htm") || Self::is_embed_player(url);
                    if is_webpage {
                        if let Some(ref sender) = tx {
                            let _ = sender.send(DownloadProgress {
                                ep_number: ep_num,
                                title: format!("Episode {:02}", ep_num),
                                downloaded_bytes: 0,
                                total_bytes: 0,
                                percent: 0.0,
                                status: format!("yt-dlp failed: {}", e),
                                is_finished: true,
                                error: Some(e.to_string()),
                            }).await;
                        }
                        return Err(e);
                    }

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
        if url.ends_with(".html") || url.ends_with(".htm") {
            return Err("Cannot download HTML webpage directly as video".into());
        }

        let resp = self.client.head(url).send().await?;
        if let Some(ct) = resp.headers().get(reqwest::header::CONTENT_TYPE).and_then(|val| val.to_str().ok()) {
            if ct.contains("text/html") || ct.contains("text/plain") {
                return Err("Server returned HTML page instead of video media".into());
            }
        }

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

        if Self::is_embed_player(url) || url.contains(".m3u8") {
            pb.set_message("Downloading via yt-dlp...");
            match self.download_with_ytdlp(url, ep_num, None).await {
                Ok(path) => {
                    pb.finish_with_message(format!("Episode {} downloaded successfully with yt-dlp", ep_num));
                    return Ok(path);
                }
                Err(e) => {
                    if url.contains(".htm") || Self::is_embed_player(url) {
                        pb.finish_with_message(format!("Episode {} failed with yt-dlp: {}", ep_num, e));
                        return Err(e);
                    }
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
        if url.ends_with(".html") || url.ends_with(".htm") {
            return Err("Cannot download HTML webpage directly as video".into());
        }

        let resp = self.client.head(url).send().await?;
        if let Some(ct) = resp.headers().get(reqwest::header::CONTENT_TYPE).and_then(|val| val.to_str().ok()) {
            if ct.contains("text/html") || ct.contains("text/plain") {
                return Err("Server returned HTML page instead of video media".into());
            }
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ytdlp_progress_known_total() {
        let line = "YTPROG|||5242880|||10485760||| 50.0%|||  2.50MiB/s|||00:02";
        let prog = Downloader::parse_ytdlp_progress(line, 1).expect("Should parse");
        assert_eq!(prog.ep_number, 1);
        assert_eq!(prog.downloaded_bytes, 5242880);
        assert_eq!(prog.total_bytes, 10485760);
        assert!((prog.percent - 50.0).abs() < 0.1);
        assert!(prog.status.contains("5.0 MB / 10.0 MB"));
        assert!(prog.status.contains("2.50MiB/s"));
        assert!(prog.status.contains("00:02"));
    }

    #[test]
    fn test_parse_ytdlp_progress_unknown_total() {
        let line = "YTPROG|||2097152|||NA|||  0.0%||| Unknown B/s|||Unknown";
        let prog = Downloader::parse_ytdlp_progress(line, 5).expect("Should parse");
        assert_eq!(prog.ep_number, 5);
        assert_eq!(prog.downloaded_bytes, 2097152);
        assert_eq!(prog.total_bytes, 0);
        assert!((prog.percent - 0.0).abs() < 0.1);
        assert!(prog.status.contains("2.0 MB — Unknown B/s ETA Unknown"));
    }

    #[test]
    fn test_parse_ytdlp_progress_invalid_line() {
        let line = "[download] Destination: episode_01.mp4";
        assert!(Downloader::parse_ytdlp_progress(line, 1).is_none());

        let line2 = "YTPROG|||1024|||not_enough_parts";
        assert!(Downloader::parse_ytdlp_progress(line2, 1).is_none());
    }
}
