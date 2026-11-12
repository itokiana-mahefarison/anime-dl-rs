use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use tokio::process::Command;

#[derive(Clone)]
pub struct Downloader {
    client: Client,
    pub output_dir: String,
    max_retries: usize,
}

impl Downloader {
    pub fn new(output_dir: String) -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()
            .unwrap_or_default();
        Self {
            client,
            output_dir,
            max_retries: 3,
        }
    }

    /// Universal download method using yt-dlp which natively supports all embed players (VOE, MOON, Streamtape, myTV, etc.)
    pub async fn download_with_ytdlp(
        &self,
        target_url: &str,
        ep_num: usize,
        mut cancel_rx: tokio::sync::watch::Receiver<()>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        fs::create_dir_all(&self.output_dir)?;
        let output_template = format!("{}/episode_{:02}.%(ext)s", self.output_dir, ep_num);
        let final_filename = format!("{}/episode_{:02}.mp4", self.output_dir, ep_num);

        let mut cmd = Command::new("yt-dlp")
            .arg("-o")
            .arg(&output_template)
            .arg("--no-playlist")
            .arg(target_url)
            .spawn()?;

        tokio::select! {
            res = cmd.wait() => {
                match res {
                    Ok(s) if s.success() => Ok(final_filename),
                    _ => Err("yt-dlp download failed or yt-dlp is not installed on the system.".into()),
                }
            }
            _ = cancel_rx.changed() => {
                let _ = cmd.kill().await;
                Err("Download cancelled".into())
            }
        }
    }

    pub async fn download_episode(
        &self,
        url: &str,
        ep_num: usize,
        pb: ProgressBar,
        cancel_rx: tokio::sync::watch::Receiver<()>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        fs::create_dir_all(&self.output_dir)?;
        let filename = format!("{}/episode_{:02}.mp4", self.output_dir, ep_num);

        // ALWAYS try yt-dlp first as requested
        pb.set_message("Downloading via yt-dlp...");
        match self.download_with_ytdlp(url, ep_num, cancel_rx.clone()).await {
            Ok(path) => {
                pb.finish_with_message(format!("Episode {} downloaded successfully with yt-dlp", ep_num));
                return Ok(path);
            }
            Err(e) => {
                if e.to_string().contains("cancelled") {
                    pb.finish_with_message("Download cancelled.");
                    return Err(e);
                }
                // If it's an embed/iframe URL, try_download cannot download HTML pages as video files.
                if url.contains("/e/") || url.contains("streamtape") || url.contains("voe") || url.contains("embed") {
                    return Err(format!("yt-dlp failed on embed URL (HTTP fallback skipped for HTML iframes): {}", e).into());
                }
                pb.set_message("yt-dlp failed, falling back to smart HTTP downloader...");
            }
        }

        for attempt in 1..=self.max_retries {
            match self.try_download(url, &filename, &pb, cancel_rx.clone()).await {
                Ok(_) => {
                    pb.finish_with_message(format!("Episode {} downloaded successfully", ep_num));
                    return Ok(filename);
                }
                Err(e) => {
                    if e.to_string().contains("cancelled") {
                        pb.finish_with_message("Download cancelled.");
                        return Err(e);
                    }
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

    async fn try_download(
        &self,
        url: &str,
        filename: &str,
        pb: &ProgressBar,
        mut cancel_rx: tokio::sync::watch::Receiver<()>,
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

        loop {
            tokio::select! {
                chunk_res = res.chunk() => {
                    match chunk_res? {
                        Some(chunk) => {
                            file.write_all(&chunk)?;
                            downloaded += chunk.len() as u64;
                            pb.set_position(downloaded);
                        }
                        None => break,
                    }
                }
                _ = cancel_rx.changed() => {
                    return Err("Download cancelled".into());
                }
            }
        }

        Ok(())
    }
}

pub fn parse_ytdlp_progress(line: &str) -> Option<(f64, String)> {
    if !line.contains("[download]") || line.contains("Destination:") {
        return None;
    }
    let trimmed = line.split("[download]").nth(1)?.trim();
    let words: Vec<&str> = trimmed.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }

    let pct_str = words.iter().find(|w| w.ends_with('%'))?;
    let pct: f64 = pct_str.trim_end_matches('%').parse().ok()?;

    if trimmed.contains(" in ") {
        let size = words
            .iter()
            .position(|&w| w == "of")
            .and_then(|idx| words.get(idx + 1))
            .copied()
            .unwrap_or("");
        let duration = words
            .iter()
            .position(|&w| w == "in")
            .and_then(|idx| words.get(idx + 1))
            .copied()
            .unwrap_or("");
        if !size.is_empty() && !duration.is_empty() {
            return Some((pct, format!("{size} in {duration}")));
        }
    }

    let mut size_parts = Vec::new();
    let mut speed = "";
    let mut eta = "";

    if let Some(of_idx) = words.iter().position(|&w| w == "of") {
        let at_idx = words
            .iter()
            .skip(of_idx + 1)
            .position(|&w| w == "at")
            .map(|rel| of_idx + 1 + rel)
            .unwrap_or(words.len());
        for &w in &words[of_idx + 1..at_idx] {
            let clean = w.trim_matches('~').trim();
            if !clean.is_empty() {
                size_parts.push(clean);
            }
        }
    }

    if let Some(at_idx) = words.iter().position(|&w| w == "at") {
        if let Some(&s) = words.get(at_idx + 1) {
            speed = s;
        }
    }

    if let Some(eta_idx) = words.iter().position(|&w| w == "ETA") {
        if let Some(&e) = words.get(eta_idx + 1) {
            let clean_eta = e.trim_end_matches(['(', ')', ',']);
            if !clean_eta.eq_ignore_ascii_case("unknown") {
                eta = clean_eta;
            }
        }
    }

    let size = size_parts.join(" ");
    let mut parts = Vec::new();
    if !size.is_empty() {
        parts.push(size);
    }
    if !speed.is_empty() {
        parts.push(speed.to_string());
    }
    if !eta.is_empty() {
        parts.push(format!("ETA {eta}"));
    }

    let status = if parts.is_empty() {
        trimmed.to_string()
    } else {
        parts.join(" | ")
    };

    Some((pct, status))
}
