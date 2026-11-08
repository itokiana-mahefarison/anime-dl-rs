# User Documentation - anime-dl-rs

Welcome to the user guide for **anime-dl-rs**, an interactive TUI and CLI anime downloader optimized for **voir-anime.to**, featuring universal **yt-dlp** integration.

---

## Getting Started

### Prerequisites
- **Rust & Cargo:** Edition 2021 or later (`rustup`).
- **yt-dlp:** Required for universal downloading from embed players (VOE, Streamtape, MOON, myTV, etc.). Ensure `yt-dlp` is installed and available in your PATH.
- **FFmpeg:** Recommended for merging video/audio streams.
- **Internet Connection:** Required for live scraping and integration tests.

### Building from Source
```bash
cargo build --release
```
The compiled binary will be located at `target/release/anime-dl-rs`.

---

## Testing Guide

To run the test suite (unit and live integration tests):
```bash
cargo test -- --nocapture
```

---

## Usage Guide

### 1. Interactive TUI Mode
Launch the application without arguments to open the terminal user interface:
```bash
cargo run --release
```

- **Navigation:** Use arrow keys or `Tab` to navigate between views.
- **Search:** Type your anime query in the search bar. Both VOSTFR and VF editions will be listed for you to choose from.
- **Settings (`/settings`):** Type `/settings` or press `s` to configure your preferred video players, priority order, and default language.

### 2. CLI Direct Download Mode
To download directly via command line by providing an anime URL:
```bash
cargo run --release -- --url "https://voir-anime.to/anime/mission-yozakura-family-2/" --output "./downloads"
```

---

## Configuration File
Your settings are stored persistently at:
`~/.config/anime-dl/config.json`
