# Technical Specification (CDC) - anime-dl-rs

## 1. Introduction
`anime-dl-rs` is a high-performance, asynchronous Rust command-line and terminal user interface (TUI) application designed to search, parse, and download anime episodes from **voir-anime.to**, built following Test-Driven Development (TDD).

---

## 2. Core Architecture & Modules

The project follows a modular Rust architecture:
- `main.rs`: Entry point, argument parsing (`clap`), and initialization (TUI or CLI mode).
- `app.rs`: Application state management for the TUI (`ratatui`).
- `ui.rs`: Rendering logic for views (Search, Episode list, Settings modal, Download progress).
- `scraper.rs`: HTTP client (`reqwest`), HTML parsing (`scraper`), handling search queries, language filtering (VOSTFR/VF), player iframe extraction, and stream resolution (`resolve_video_url`).
- `downloader.rs`: Smart downloader supporting HTTP `Range` requests for resumable downloads, concurrent downloads via `tokio`, and `indicatif` progress bars.
- `config.rs`: Persistent configuration (`~/.config/anime-dl/config.json`) for language preferences and prioritized player lists.

---

## 3. Testing Strategy & Implementation (TDD)

The project includes unit tests and live integration tests:
- **Unit Tests:**
  - Configuration defaults and JSON persistence (`config.rs`).
  - Episode number regex extraction and search HTML parsing (`scraper.rs`).
  - TUI state navigation, settings toggles, and cursor bounds (`app.rs`).
- **Live Integration Tests (`tests/scraper_integration.rs`):**
  - Live query testing against `voir-anime.to` for search, episode listing, player iframe resolution, and direct video stream extraction.

### Running Tests
```bash
cargo test -- --nocapture
```
