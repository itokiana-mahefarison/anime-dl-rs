# Anime-DL (Rust TUI & CLI)

A high-performance, asynchronous anime downloader and TUI client for **voir-anime.to**, built in Rust following Test-Driven Development (TDD) and best engineering practices.

---

## Features

- **Dual Search Mode:**
  - **Global Search:** Search for anime across the platform, displaying all available language versions (VOSTFR & VF) for manual selection.
  - **Direct URL Mode:** Provide the direct anime link for instant parsing.
- **Smart Player Fallback:** Supports multiple video players (`Streamtape`, `myTV`, `MOON`, `VOE`) with automatic failover in case of blockers or captchas.
- **Interactive TUI & Settings (`/settings`):**
  - Built with `ratatui` and `crossterm`.
  - Configurable language preference (VOSTFR / VF).
  - Interactive player priority reordering with checkboxes.
- **Smart Downloader:**
  - Episode range selection.
  - Concurrent asynchronous downloads using `tokio`.
  - Resumable downloads via HTTP `Range` requests.
  - Real-time progress bars (`indicatif`).

---

## Prerequisites

- **Rust & Cargo:** Edition 2021 or later (`rustup`).
- **Internet Connection:** Required for live scraping and integration tests against `voir-anime.to`.

---

## Installation & Building

```bash
git clone https://github.com/your-username/anime-dl-rs.git
cd anime-dl-rs
cargo build --release
```

---

## Testing

The project includes a robust suite of both unit tests and live integration tests following TDD principles.

### Running Tests
To run all tests (unit tests and live integration tests):
```bash
cargo test -- --nocapture
```

### Implemented Test Suite:
1. **Unit Tests (`config.rs`):**
   - `test_default_config`: Validates default language (VOSTFR) and default preferred players order.
   - `test_config_serialization`: Ensures correct JSON serialization and deserialization of preferences.
2. **Unit Tests (`scraper.rs`):**
   - `test_extract_episode_number`: Tests robust regex extraction of episode numbers from various formats (VOSTFR/VF slugs).
   - `test_parse_search_html`: Tests HTML parsing of search results and language annotations.
3. **Unit Tests (`app.rs`):**
   - `test_app_initial_state`: Verifies initial TUI view and empty search query.
   - `test_toggle_settings`: Tests switching between main views and the `/settings` modal.
   - `test_settings_cursor_movement`: Validates cursor bounds and navigation.
4. **Live Integration Tests (`tests/scraper_integration.rs`):**
   - `test_live_search_voir_anime`: Performs a live query against `voir-anime.to` for search results.
   - `test_live_get_episodes`: Fetches and validates live episode lists for a series.
   - `test_live_player_iframe_extraction`: Resolves player iframes and extracts direct video stream URLs (`.mp4` / `.m3u8` / `robotlink`).
   - `test_downloader_initialization`: Verifies smart downloader setup.

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
