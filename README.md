# Anime-DL (Rust TUI & CLI)

A high-performance, asynchronous anime downloader and TUI client for **voir-anime.to**, built in Rust with **yt-dlp** backend integration for universal player support.

---

## Features

- **Dual Search & Direct URL Mode:**
  - **Global Search:** Search for anime across the platform, displaying all available language versions (VOSTFR & VF) for manual selection.
  - **Direct URL Support:** Paste or type direct anime links anywhere (in the TUI search bar or via CLI flags) for instant parsing.
- **Universal Player & yt-dlp Support:**
  - Supports all embed players (`Streamtape`, `VOE`, `MOON`, `myTV`) with native integration of **yt-dlp** for robust stream extraction and downloading across all third-party hosts.
  - **Automatic Player Fallback:** If a player link returns an error (e.g. 403 Forbidden), the client automatically tries alternative preferred players.
- **Interactive TUI & Settings (`/settings`):**
  - Built with `ratatui` and `crossterm` featuring modern rounded borders, ASCII art branding, and responsive layout navigation.
  - Configurable language preference (VOSTFR / VF).
  - Interactive player priority reordering with checkboxes.
- **Smart Downloader:**
  - Concurrent asynchronous downloads (batches of 3 in parallel).
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

## Usage Guide & Examples

### 1. Interactive TUI Mode (Default)
Launch the application in interactive TUI mode:
```bash
cargo run --release
```
- **Search:** Type your query (e.g., `Kimetsu no Yaiba`) or paste a direct `voir-anime.to` URL straight into the search box and press `Enter`.
- **Navigation:** Use `↑` / `↓` arrows to navigate results or episodes, and `Enter` to select/download.
- **Settings:** Type `/s` or `/settings` in the search bar to configure preferred player priorities and language preferences.

### 2. Direct URL TUI Loading
Launch the TUI pre-loaded with episodes from a specific anime URL:
```bash
cargo run --release -- --url "https://voir-anime.to/anime/kimetsu-no-yaiba-vf/" --output "/home/itokiana/anime/Demon_Slayer/S01/"
```

### 3. Non-Interactive CLI Batch Download Mode (`--cli`)
Download all episodes of an anime in non-interactive batch mode (in batches of 3 in parallel, with automatic player fallback and `yt-dlp` priority):
```bash
cargo run --release -- --url "https://voir-anime.to/anime/kimetsu-no-yaiba-vf/" --output "/home/itokiana/anime/Demon_Slayer/S01/" --cli
```

---

## License

This project is licensed under the MIT License.
