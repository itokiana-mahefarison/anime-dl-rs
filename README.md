# Anime-DL (Rust TUI & CLI)

A high-performance, asynchronous anime downloader and TUI client for **voir-anime.to** and **anime-sama.to**, built in Rust with **yt-dlp** backend integration for universal player support.

---

## Features

- **Multi-Provider Architecture:**
  - Support for **Voir-Anime** (`voir-anime.to`) and **Anime-Sama** (`anime-sama.to`).
  - Toggle active provider instantly with `Ctrl+P`.
- **Dual Search Mode:**
  - **Global Search:** Search for anime across the active platform, displaying all available language versions (VOSTFR & VF).
  - **Direct URL Mode:** Provide the direct anime link for instant parsing.
- **Universal Player & yt-dlp Support:**
  - Supports embed players (`Vidmoly`, `Uqload`, `Streamtape`, `VOE`, `MOON`, `myTV`) with native integration of **yt-dlp** for robust stream extraction and downloading across third-party hosts.
- **Non-blocking Interactive TUI:**
  - Built with `ratatui` and `crossterm`.
  - Non-blocking background downloads with live progress gauge via Tokio MPSC channels.
  - Dedicated search input focus and results navigation.
  - Quick settings modal (`Ctrl+S`) to configure language preference and reorder player priorities (`+/-`).
- **Smart Downloader & CLI:**
  - Episode range selection (`--range 1-12`).
  - Resumable HTTP `Range` downloads with retries.

---

## Prerequisites

- **Rust & Cargo:** Edition 2021 or later (`rustup`).
- **yt-dlp:** Required for universal downloading from embed players (Vidmoly, Uqload, VOE, Streamtape, MOON, etc.). Ensure `yt-dlp` is installed and available in your PATH.
- **FFmpeg:** Recommended for merging video/audio streams.
- **Internet Connection:** Required for live scraping and integration tests.

---

## Installation & Building

```bash
cargo build --release
```

---

## Testing

To run the complete test suite:
```bash
cargo test -- --nocapture
```

---

## Usage

### 1. Interactive TUI Mode
```bash
cargo run --release
```

- **Search:** Type your query and press `Enter`.
- **Navigation:** Press `Tab` or `Down` to browse results. Press `Tab` or `/` to return to the search query.
- **Episodes:** Press `Enter` on an anime to view episodes. Press `Enter` on an episode to start downloading in the background.
- **Provider:** Press `Ctrl+P` to switch between `voir-anime` and `anime-sama`.
- **Settings:** Press `Ctrl+S` to open the configuration modal (`l` for language, `+`/`-` to reorder players).
- **Quit / Back:** Press `Esc`.

### 2. CLI Direct Download Mode
```bash
# Download all episodes
cargo run --release -- --url "https://voir-anime.to/anime/mission-yozakura-family-2/" --output "./downloads"

# Download a range of episodes (e.g. episodes 1 to 5)
cargo run --release -- --url "https://anime-sama.to/catalogue/naruto/saison1/vostfr/" --output "./downloads" --range 1-5
```

---

## License

This project is licensed under the MIT License.
