# User Documentation - anime-dl-rs

Welcome to the user guide for **anime-dl-rs**, an interactive TUI and CLI anime downloader supporting **voir-anime.to** and **anime-sama.to**, featuring universal **yt-dlp** integration.

---

## Getting Started

### ⚡ Quick Installation (One-Liner)

#### Linux & macOS:
```bash
curl -fsSL https://raw.githubusercontent.com/itokiana/anime-dl-rs/master/install.sh | bash
```

#### Windows (PowerShell):
```powershell
irm https://raw.githubusercontent.com/itokiana/anime-dl-rs/master/install.ps1 | iex
```

#### Cargo:
```bash
cargo install --git https://github.com/itokiana/anime-dl-rs.git
```

---

### Prerequisites & Manual Build

---

## Testing Guide

To run the complete test suite:
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

- **Search:** Type your anime query in the search bar and press `Enter`.
- **Browsing Results:** Use `Down` or `Tab` to navigate through results. Press `Tab` or `/` to return to editing your search query.
- **Episodes:** Press `Enter` on an anime to fetch and view its episode list.
- **Background Download:** Press `Enter` on an episode to start downloading. The download runs in a background thread and updates a live progress gauge in the TUI without freezing the interface.
- **Switch Provider:** Press `Ctrl+P` at any time to switch between `voir-anime` and `anime-sama`.
- **Settings:** Press `Ctrl+S` to open the settings modal. Press `l` to toggle default language (`VOSTFR`/`VF`), and `+`/`-` to reorder preferred video players.
- **Back / Quit:** Press `Esc`.

### 2. CLI Direct Download Mode
Download directly via command line by providing an anime URL:
```bash
# Download all episodes
cargo run --release -- --url "https://voir-anime.to/anime/mission-yozakura-family-2/" --output "./downloads"

# Download with episode range (e.g. 1 to 5)
cargo run --release -- --url "https://anime-sama.to/catalogue/naruto/saison1/vostfr/" --output "./downloads" --range 1-5
```

---

## Configuration File
Your settings are stored persistently at:
`~/.config/anime-dl/config.json`
