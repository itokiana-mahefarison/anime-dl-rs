# Anime-DL (Rust TUI & CLI)

A high-performance, asynchronous anime downloader and TUI client for **voir-anime.to**, built in Rust with **yt-dlp** backend integration for universal player support.

---

## Features

- **Dual Search Mode:**
- **Global Search:** Search for anime across the platform, displaying all available language versions (VOSTFR & VF) for manual selection.
- **Direct URL Mode:** Provide the direct anime link for instant parsing.
- **Universal Player & yt-dlp Support:**
- Supports all embed players (`Streamtape`, `VOE`, `MOON`, `myTV`) with native integration of **yt-dlp** for robust stream extraction and downloading across all third-party hosts.
- **Interactive TUI & Settings (`/settings`):**
- Built with `ratatui` and `crossterm`.
- Configurable language preference (VOSTFR / VF).
- Interactive player priority reordering with checkboxes.
- **Smart Downloader:**
- Episode range selection.
- Concurrent asynchronous downloads using `tokio`.
- Resumable HTTP `Range` downloads & progress bars (`indicatif`).

---

## Prerequisites

- **Rust & Cargo:** Edition 2021 or later (`rustup`).
- **yt-dlp:** Required for universal downloading from embed players (VOE, Streamtape, MOON, etc.). Ensure `yt-dlp` is installed and available in your PATH.
- **FFmpeg:** Recommended for merging video/audio streams.
- **Internet Connection:** Required for live scraping and integration tests.

---

## Installation & Building

```bash
git clone https://github.com/your-username/anime-dl-rs.git
cd anime-dl-rs
cargo build --release
```

---

## Testing

To run the complete test suite (unit and live integration tests):
```bash
cargo test -- --nocapture
```

---

## Usage

Run the application in interactive TUI mode:
```bash
cargo run --release
```

For direct downloads via CLI:
```bash
cargo run --release -- --url "https://voir-anime.to/anime/mission-yozakura-family-2/" --output "./downloads"
```

---

## License

This project is licensed under the MIT License.
