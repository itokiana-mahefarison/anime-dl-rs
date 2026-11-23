# AGENT.md - Persistent Project Context for `anime-dl-rs`

This document serves as the persistent memory and technical briefing for AI agents opening a new session to work on or maintain **anime-dl-rs**.

---

## 1. Project Overview
- **Name:** `anime-dl-rs`
- **Description:** A high-performance, asynchronous Rust anime downloader and Terminal User Interface (TUI) client for **voiranime.cx** and **anime-sama.to**, built following Test-Driven Development (TDD) principles.
- **Target Sites:** `voiranime.cx` (WordPress / Madara theme) and `anime-sama.to` (single-page season structure with Vidmoly/Uqload/VOE).
- **Key Features:**
  - Multi-provider support (`voir-anime` and `anime-sama` switchable via `Ctrl+P`).
  - Dual search mode (Global platform search & Direct URL mode).
  - Universal player support via **yt-dlp** integration (Vidmoly, Uqload, VOE, MOON, Streamtape, myTV) with HTTP `Range` smart fallback.
  - Interactive non-blocking TUI (`ratatui` + `crossterm`) with search view, dedicated results navigation, episode list, live progress gauge, and settings modal (`Ctrl+S`).
  - CLI direct mode with episode range selection (`--range 1-12`).
  - Persistent user configuration (`~/.config/anime-dl/config.json`).

---

## 2. Architecture & Directory Structure

```
projects/anime-dl-rs/
├── Cargo.toml               # Dependencies (tokio, reqwest, ratatui, crossterm, regex, serde, url)
├── Cargo.lock
├── README.md                # Public project documentation
├── CDC.md                   # Technical specification
├── docs.md                  # User guide
├── AGENT.md                 # This persistent AI briefing file
├── src/
│   ├── lib.rs               # Library root exporting modules
│   ├── main.rs              # CLI argument parser (clap) & async TUI event loop with mpsc channels
│   ├── app.rs               # TUI state machine, navigation, view modes & download progress struct
│   ├── ui.rs                # Ratatui rendering engine (focus states, episode list, progress gauge)
│   ├── config.rs            # Persistent JSON configuration module
│   ├── downloader.rs        # yt-dlp wrapper & async HTTP Range downloader with MPSC progress
│   └── providers/
│       ├── mod.rs           # ProviderEnum abstraction (AnimeItem, EpisodeItem)
│       ├── voir_anime.rs    # voiranime.cx HTML scraper and stream resolver
│       └── anime_sama.rs    # anime-sama.to season parser and embed resolver
└── tests/
    └── scraper_integration.rs # Live integration tests (search, episodes, iframes, stream resolution)
```

---

## 3. Prerequisites & System Dependencies
- **Rust & Cargo:** Edition 2021+ (`rustup`).
- **yt-dlp:** Required for universal media extraction and downloading across all player embeds. Must be installed and in `PATH`.
- **FFmpeg:** Recommended for merging video/audio streams.
- **Internet Connection:** Required for live scraping and integration tests.

---

## 4. Quick Start Commands

### Run TUI Mode
```bash
cargo run
```
- Type any query in the search bar and press `Enter`.
- Use `Tab` or `Down` to navigate results.
- Press `Enter` to open episodes, then `Enter` on an episode to trigger background download.
- Press `Ctrl+P` to switch provider (Voir-Anime <-> Anime-Sama).
- Press `Ctrl+S` to open Settings.

### Run CLI Direct Download Mode
```bash
cargo run -- --url "https://voiranime.cx/anime/mission-yozakura-family-2/" --output "./downloads" --range 1-5
```

### Run Test Suite
```bash
cargo test -- --nocapture
```
