# User Documentation - anime-dl-rs

Welcome to the user guide for **anime-dl-rs**, a high-performance interactive TUI and CLI anime downloader optimized for **voir-anime.to**, featuring universal **yt-dlp** integration, automatic player fallback, and 3-way concurrent batch downloading.

---

## Getting Started

### Prerequisites
- **Rust & Cargo:** Edition 2021 or later (`rustup`).
- **yt-dlp:** Required for universal downloading from embed players (VOE, Streamtape, MOON, myTV, etc.). Ensure `yt-dlp` is installed and available in your PATH (`yt-dlp --version`).
- **FFmpeg:** Recommended for merging video/audio streams.
- **Internet Connection:** Required for live scraping and integration tests.

### Building from Source
```bash
cargo build --release
```
The compiled binary will be located at `target/release/anime-dl-rs`.

---

## Testing Guide

To run the complete test suite (unit and live integration tests):
```bash
cargo test -- --nocapture
```

---

## Usage Guide & Options

### 1. Interactive TUI Mode (Default)
Launch the application without arguments to open the terminal user interface:
```bash
cargo run --release
```
- **Search & Direct URLs:** Type keywords in the search bar (e.g. `Mission Yozakura Family`) or paste a direct `voir-anime.to` URL (e.g. `https://voir-anime.to/anime/mission-yozakura-family-2/`) directly into the search box and press `Enter` to instantly view episodes.
- **Navigation:** Use `↑` / `↓` arrow keys to navigate lists and press `Enter` to select an anime or download an episode. Press `Esc` to go back.
- **Settings Modal:** Type `/s` or `/settings` in the search bar to open settings. Use `↑` / `↓` to select players, `+` / `-` to reorder player priority, and `l` to toggle language preference (VOSTFR / VF).

### 2. Direct URL TUI Loading
Launch the interactive TUI pre-loaded with episodes from a specific anime URL, specifying a custom output folder:
```bash
cargo run --release -- --url "https://voir-anime.to/anime/kimetsu-no-yaiba-vf/" --output "/home/itokiana/anime/Demon_Slayer/S01/"
```

### 3. Non-Interactive CLI Batch Download Mode (`--cli`)
To download all episodes of an anime in non-interactive batch mode directly from the terminal (downloading up to 3 episodes in parallel, trying `yt-dlp` first, and falling back across preferred players automatically):
```bash
cargo run --release -- --url "https://voir-anime.to/anime/kimetsu-no-yaiba-vf/" --output "/home/itokiana/anime/Demon_Slayer/S01/" --cli
```

---

## Configuration File
Your persistent configuration (language preference and player priority) is stored at:
`~/.config/anime-dl/config.json`
