#!/usr/bin/env bash
set -e

# ==============================================================================
# anime-dl-rs Installer for Linux & macOS
# One-liner: curl -fsSL https://raw.githubusercontent.com/itokiana/anime-dl-rs/master/install.sh | bash
# ==============================================================================

BOLD='\033[1m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

REPO_URL="${REPO_URL:-https://github.com/itokiana/anime-dl-rs.git}"
APP_NAME="anime-dl-rs"

printf "${CYAN}${BOLD}"
cat << "EOF"
               _                     _ _                
  __ _ _ _  (_)_ __  ___ ___ __| | |___ _ _ ___ 
 / _` | ' \ | | '  \/ -_)___/ _` | |___| '_(_-< 
 \__,_|_||_|_|_|_|_\___|   \__,_|_|   |_| /__/ 
EOF
printf "${NC}\n"
printf "${BOLD}Installing %s...${NC}\n\n" "$APP_NAME"

# 1. Check & Install Rust / Cargo if missing
if ! command -v cargo >/dev/null 2>&1; then
    printf "${YELLOW}● Cargo/Rust is not detected on your system.${NC}\n"
    printf "${CYAN}Installing Rust via rustup...${NC}\n"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
fi

# 2. Check for yt-dlp & ffmpeg
if ! command -v yt-dlp >/dev/null 2>&1; then
    printf "${YELLOW}⚠ Warning: 'yt-dlp' was not found in your PATH.${NC}\n"
    printf "  Please install it for full stream extraction support:\n"
    printf "  - Ubuntu/Debian : sudo apt install yt-dlp\n"
    printf "  - Arch Linux    : sudo pacman -S yt-dlp\n"
    printf "  - macOS         : brew install yt-dlp\n"
    printf "  - Python pip    : pip install yt-dlp\n\n"
fi

if ! command -v ffmpeg >/dev/null 2>&1; then
    printf "${YELLOW}⚠ Warning: 'ffmpeg' was not found in your PATH.${NC}\n"
    printf "  Please install it for optimal video/audio merging.\n\n"
fi

# 3. Determine Installation Target Directory
INSTALL_DIR=""
if [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

# 4. Clone and Build
TMP_DIR="$(mktemp -d -t anime-dl-build-XXXXXX)"
printf "${CYAN}● Cloning repository into temporary workspace...${NC}\n"
git clone --depth 1 "$REPO_URL" "$TMP_DIR" >/dev/null 2>&1

printf "${CYAN}● Compiling %s (release build with Cargo)...${NC}\n" "$APP_NAME"
(
    cd "$TMP_DIR"
    cargo build --release
)

# 5. Move binary to installation path
TARGET_BIN="$TMP_DIR/target/release/$APP_NAME"
if [ ! -f "$TARGET_BIN" ]; then
    printf "${RED}✖ Build failed: %s not found.${NC}\n" "$TARGET_BIN"
    rm -rf "$TMP_DIR"
    exit 1
fi

printf "${CYAN}● Installing binary to %s/%s...${NC}\n" "$INSTALL_DIR" "$APP_NAME"
cp "$TARGET_BIN" "$INSTALL_DIR/$APP_NAME"
chmod +x "$INSTALL_DIR/$APP_NAME"

# Clean temporary directory
rm -rf "$TMP_DIR"

# 6. Verify PATH
if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
    printf "\n${YELLOW}⚠ Notice: %s is not currently in your PATH.${NC}\n" "$INSTALL_DIR"
    printf "Add the following line to your ~/.bashrc or ~/.zshrc:\n"
    printf "  export PATH=\"%s:\$PATH\"\n" "$INSTALL_DIR"
fi

printf "\n${GREEN}${BOLD}✔ %s successfully installed!${NC}\n\n" "$APP_NAME"
printf "Usage:\n"
printf "  ${BOLD}%s${NC}                     # Launch modern interactive TUI\n" "$APP_NAME"
printf "  ${BOLD}%s --help${NC}              # View CLI options\n" "$APP_NAME"
printf "  ${BOLD}%s --url <ANIME_URL>${NC}    # Direct download\n\n" "$APP_NAME"
