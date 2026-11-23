<div align="center">

```text
               _                     _ _                
  __ _ _ _  (_)_ __  ___ ___ __| | |___ _ _ ___ 
 / _` | ' \ | | '  \/ -_)___/ _` | |___| '_(_-< 
 \__,_|_||_|_|_|_|_\___|   \__,_|_|   |_| /__/ 
```

# anime-dl-rs

### High-performance, asynchronous Anime Downloader & TUI Client for Rust lovers.

[![Rust](https://img.shields.io/badge/Rust-2021%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Tokio](https://img.shields.io/badge/Async-Tokio-blue.svg?style=flat-square)](https://tokio.rs/)
[![Ratatui](https://img.shields.io/badge/TUI-Ratatui-green.svg?style=flat-square)](https://ratatui.rs/)
[![Backend](https://img.shields.io/badge/Backend-yt--dlp-red.svg?style=flat-square)](https://github.com/yt-dlp/yt-dlp)
[![License](https://img.shields.io/badge/License-MIT-purple.svg?style=flat-square)](LICENSE)

[**Explore Features**](#-key-features) •
[**Quick Start**](#-quick-start) •
[**TUI Controls**](#-interactive-tui) •
[**CLI Usage**](#-cli-power-mode) •
[**Architecture**](#-supported-providers--players)

</div>

---

## ⚡ The Modern Terminal Anime Experience

**`anime-dl-rs`** is built from the ground up for speed, elegance, and resilience. Whether you prefer a blazing-fast command-line tool with episode range filtering or a keyboard-centric, distraction-free **Terminal User Interface (TUI)** inspired by [*binsider*](https://github.com/orhun/binsider), `anime-dl-rs` delivers full control over your anime downloads.

```text
╭─┤ anime-dl-rs v0.1.0 ├───────────────────────────────────────────────────────╮
│ [1] Search │ [2] Episodes │ [3] Settings      ● anime-sama │ VOSTFR │ ./downloads│
│──────────────────────────────────────────────────────────────────────────────│
│ ╭── Search Anime (Type query & press Enter) ───────────────────────────────╮ │
│ │ ❯ Solo Leveling█                                                         │ │
│ ╰──────────────────────────────────────────────────────────────────────────╯ │
│ ╭── Results (2) ──────────────────╮ ╭── Anime Information ─────────────────╮ │
│ │ ▶ [VOSTFR] Solo Leveling        │ │ Title     : Solo Leveling (VOSTFR)   │ │
│ │   [VF]     Solo Leveling        │ │ Language  : VOSTFR                   │ │
│ │                                 │ │ Target URL: https://anime-sama.to/...│ │
│ │                                 │ │ Action    : Press Enter for episodes │ │
│ ╰─────────────────────────────────╯ ╰──────────────────────────────────────╯ │
│ ╭── Download Progress ─────────────────────────────────────────────────────╮ │
│ │ Episode 01 ➔ 45.8% [112.4 MB / 245.3 MB (45.8%)]                         │ │
│ ╰──────────────────────────────────────────────────────────────────────────╯ │
│                                                                              │
│ [Enter → Select]  [Tab → Switch Focus]  [Ctrl+P → Provider]  [Ctrl+S → Set]  │
╰──────────────────────────────────────────────────────────────────────────────╯
```

---

## ✨ Key Features

| Feature | Description |
| :--- | :--- |
| **Modular Multi-Provider** | Native support for **VoirAnime** (`voiranime.cx`) and **Anime-Sama** (`anime-sama.to`). Seamlessly switch between sources on the fly. |
| **Binsider-Inspired TUI** | Beautiful rounded cards, tabbed views, search prompt cursor, real-time metadata inspector, and status pills. |
| **Non-blocking Downloads** | Zero GUI freezes. Background tasks stream progress via `tokio::mpsc` channels directly into an in-terminal gauge. |
| **Universal Video Backend** | Deep integration with **`yt-dlp`** to bypass third-party player protections across all major embed hosts. |
| **Smart CLI Range Filter** | Download exactly what you want via CLI: full seasons, single episodes, or ranges (`--range 1-12`). |
| **Resumable HTTP Engine** | Built-in fallback downloader with HTTP `Range` request support, automatic retry loops, and integrity verification. |
| **Persistent Configuration** | Save your preferred audio language (`VOSTFR` / `VF`) and reorder player priorities (`~/.config/anime-dl/config.json`). |

---

## 🚀 Quick Start

### ⚡ 1. One-Liner Instant Installation (Pre-compiled Binaries)

Install and setup `anime-dl-rs` in seconds without needing Rust, Cargo, or Git:

#### Linux & macOS:
```bash
curl -fsSL https://raw.githubusercontent.com/itokiana/anime-dl-rs/master/install.sh | bash
```

#### Windows (PowerShell):
```powershell
irm https://raw.githubusercontent.com/itokiana/anime-dl-rs/master/install.ps1 | iex
```

> [!TIP]
> The installer automatically fetches the latest pre-compiled static binary from [GitHub Releases](https://github.com/itokiana/anime-dl-rs/releases).

#### Via Cargo (for Rust developers):
```bash
cargo install --git https://github.com/itokiana/anime-dl-rs.git
```

---

### 📦 2. Runtime Dependencies & Manual Build

#### Runtime Dependencies
To extract streams and merge formats, ensure you have:
- **yt-dlp**: `sudo apt install yt-dlp` (Linux) or `winget install yt-dlp` (Windows)
- **FFmpeg**: `sudo apt install ffmpeg` (Linux) or `winget install Gyan.FFmpeg` (Windows)

#### Building from Source (Optional)
If you prefer building locally:
- **Rust toolchain** (1.75+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

```bash
git clone https://github.com/itokiana/anime-dl-rs.git
cd anime-dl-rs
cargo build --release
```

The compiled binary will be placed at `target/release/anime-dl-rs`.

---

## 🖥️ Interactive TUI

Launch the full interactive experience with no arguments:

```bash
cargo run --release
```

### Keyboard Cheat Sheet

| Key | Context | Action |
| :--- | :--- | :--- |
| `Enter` | **Search** | Perform search query & switch to results |
| `Tab` / `Down` | **Search** | Transfer focus to results list |
| `Tab` / `/` | **Results** | Return focus to search bar to edit query |
| `Up` / `Down` | **Results / Episodes** | Navigate item list |
| `Enter` | **Results** | Fetch and open episode list |
| `Enter` | **Episodes** | Trigger non-blocking background download |
| `1` / `2` / `3` | **Global** | Jump directly between Tabs (`Search`, `Episodes`, `Settings`) |
| `Ctrl + P` | **Global** | Toggle provider (**Voir-Anime** ↔ **Anime-Sama**) |
| `Ctrl + S` | **Global** | Toggle settings modal |
| `+` / `-` | **Settings** | Reorder video player priorities |
| `l` | **Settings** | Toggle default audio language (`VOSTFR` ↔ `VF`) |
| `Esc` / `q` | **Global** | Go back / Exit application |

---

## ⚡ CLI Power Mode

Prefer scriptable or headless downloads? Use CLI direct download mode:

### Auto-Detected Provider URL
`anime-dl-rs` inspects the URL domain and automatically selects the matching engine:

```bash
# Download from Voir-Anime
cargo run --release -- --url "https://voiranime.cx/anime/mission-yozakura-family-2/"

# Download from Anime-Sama
cargo run --release -- --url "https://anime-sama.to/catalogue/naruto/saison1/vostfr/"
```

### Episode Range Filtering (`--range`)
Download specific parts of a series without manual intervention:

```bash
# Specific episode range (episodes 1 to 12)
cargo run --release -- --url "https://anime-sama.to/catalogue/naruto/saison1/vostfr/" --range 1-12

# Single episode (episode 5)
cargo run --release -- --url "https://anime-sama.to/catalogue/naruto/saison1/vostfr/" --range 5

# Open-ended range (from episode 10 to the end)
cargo run --release -- --url "https://anime-sama.to/catalogue/naruto/saison1/vostfr/" --range 10-

# Specify custom output folder
cargo run --release -- --url "https://anime-sama.to/catalogue/naruto/saison1/vostfr/" --output "/mnt/media/anime"
```

---

## 🌐 Supported Providers & Players

### Providers
- **VoirAnime** (`voiranime.cx`) — Multi-page WordPress / wp-manga scraper with automatic language variant detection.
- **Anime-Sama** (`anime-sama.to`) — Single-Page season scraper with direct player embed extraction.

### Video Players & Embeds
`anime-dl-rs` prioritizes players based on your configuration and resolves them through native `yt-dlp` integration:
- 🟢 **Vidmoly / Ansembed**
- 🟢 **VOE / VOEmbed**
- 🟢 **Uqload**
- 🟢 **Streamtape**
- 🟢 **MOON / myTV / Sendvid / Sibnet**

---

## ⚙️ Configuration

Your preferences are persistently saved in standard JSON at:
`~/.config/anime-dl/config.json`

Player priority is configured independently per provider:

```json
{
  "default_language": "VOSTFR",
  "provider": "AnimeSama",
  "voir_anime_players": [
    "Stape",
    "myTV",
    "MOON",
    "VOE"
  ],
  "anime_sama_players": [
    "Vidmoly",
    "VOE",
    "Uqload",
    "Sendvid",
    "Sibnet",
    "OneUpload",
    "Streamtape"
  ]
}
```

---

## 🧪 Testing

The repository maintains an automated test suite combining unit tests and live integration scraping:

```bash
cargo test -- --nocapture
```

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
