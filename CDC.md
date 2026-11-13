# Technical Specification (CDC) - anime-dl-rs

## 1. Introduction
`anime-dl-rs` is a high-performance, asynchronous Rust command-line and terminal user interface (TUI) application designed to search, parse, and download anime episodes from **voir-anime.to** and **anime-sama.to**, integrating **yt-dlp** for universal media extraction and downloading across all player types (Vidmoly, Uqload, VOE, Streamtape, MOON, myTV).

---

## 2. Core Architecture & Modules

The project follows a modular Rust architecture:
- `main.rs`: Entry point, argument parsing (`clap`), background task spawning (`tokio::spawn`), and TUI event loop.
- `app.rs`: Application state management for the TUI (`ratatui`), view states (`Search`, `AnimeList`, `Episodes`, `Settings`), and active download progress tracking.
- `ui.rs`: Rendering logic for views (Search with focus indicators, Episode list, Settings modal, live download progress gauge).
- `providers/`: Modular provider architecture with enum-based dispatch:
  - `mod.rs`: `ProviderEnum`, `AnimeItem`, `EpisodeItem`.
  - `voir_anime.rs`: WordPress / wp-manga HTML scraper for `voir-anime.to`.
  - `anime_sama.rs`: Single-page season parser, select/button extractor, and embed resolver for `anime-sama.to`.
- `downloader.rs`: Non-blocking smart downloader supporting **yt-dlp** backend integration for universal player support alongside HTTP `Range` resumable downloads and MPSC progress streaming.
- `config.rs`: Persistent configuration (`~/.config/anime-dl/config.json`) for language preferences (`VOSTFR`/`VF`), active provider, and prioritized player lists.

---

## 3. Testing Strategy & Implementation (TDD)

The project includes unit tests and live integration tests:
- **Unit Tests:**
  - Configuration defaults and JSON persistence (`config.rs`).
  - Episode range parsing and filtering (`main.rs`).
  - TUI state navigation, settings toggles, and cursor bounds (`app.rs`).
- **Live Integration Tests (`tests/scraper_integration.rs`):**
  - Live query testing against `voir-anime.to` and `anime-sama.to` for search, episode listing, player iframe resolution, and direct video stream extraction.

### Running Tests
```bash
cargo test -- --nocapture
```
