# AGENT.md - Persistent Project Context for `anime-dl-rs`

This document serves as the persistent memory and technical briefing for AI agents opening a new session to work on or maintain **anime-dl-rs**.

---

## 1. Project Overview
- **Name:** `anime-dl-rs`
- **Description:** A high-performance, asynchronous Rust anime downloader and Terminal User Interface (TUI) client for **voir-anime.to**, built following Test-Driven Development (TDD) principles.
- **Target Site:** `voir-anime.to` (WordPress / Madara theme).
- **Key Features:**
  - Dual search mode (Global platform search & Direct URL mode).
  - Universal player support via **yt-dlp** integration (VOE, MOON, Streamtape, myTV) with HTTP `Range` smart fallback.
  - Interactive TUI (`ratatui` + `crossterm`) with search view, episode list, and `/settings` modal (language preference VOSTFR/VF & player priority reordering).
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
│   ├── main.rs              # CLI argument parser (clap) & TUI event loop
│   ├── app.rs               # TUI state machine, navigation & views logic
│   ├── ui.rs                # Ratatui rendering engine
│   ├── scraper.rs           # voir-anime.to HTML scraper, episode parser, iframe & stream resolver
│   └── downloader.rs        # yt-dlp wrapper & smart HTTP Range resumable downloader
└── tests/
    └── scraper_integration.rs # Live integration tests (search, episodes, iframes, stream resolution)
```

---

## 3. Prerequisites & System Dependencies
- **Rust & Cargo:** Edition 2021+ (`rustup`).
- **yt-dlp:** Required for universal media extraction and downloading across all player embeds (VOE, Streamtape, MOON, myTV). Must be installed and in `PATH`.
- **FFmpeg:** Recommended for merging video/audio streams.
- **Internet Connection:** Required for live scraping and integration tests.

---

## 4. Quick Start Commands

### Run TUI Mode
```bash
cargo run --manifest-path projects/anime-dl-rs/Cargo.toml
```

### Run CLI Direct Download Mode (Custom Output Folder)
```bash
cargo run --manifest-path projects/anime-dl-rs/Cargo.toml -- --url "https://voir-anime.to/anime/mission-yozakura-family-2/" --output "./downloads"
```

### Run Test Suite (Unit & Live Integration Tests)
```bash
cargo test --manifest-path projects/anime-dl-rs/Cargo.toml -- --nocapture
```

---

## 5. Development Guidelines & Conventions
- **Language:** Rust (async/await via Tokio).
- **Testing:** Always write unit tests for modules and ensure live integration tests pass.
- **Git:** Maintain clean, atomic commits separated by feature/module/test rather than bulk commits.
- **Error Handling:** Use idiomatic Rust error handling (`Result`, `Box<dyn std::error::Error>`, `thiserror` / `anyhow`).
