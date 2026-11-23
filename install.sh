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

REPO="${REPO:-itokiana/anime-dl-rs}"
APP_NAME="anime-dl-rs"
VERSION="${VERSION:-latest}"

printf "${CYAN}${BOLD}"
cat << "EOF"
               _                     _ _                
  __ _ _ _  (_)_ __  ___ ___ __| | |___ _ _ ___ 
 / _` | ' \ | | '  \/ -_)___/ _` | |___| '_(_-< 
 \__,_|_||_|_|_|_|_\___|   \__,_|_|   |_| /__/ 
EOF
printf "${NC}\n"
printf "${BOLD}Installing %s...${NC}\n\n" "$APP_NAME"

# 1. Detect Operating System and Architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
    linux)
        case "$ARCH" in
            x86_64)
                ASSET_ARCH="linux-x86_64"
                ;;
            aarch64|arm64)
                ASSET_ARCH="linux-arm64"
                ;;
            *)
                printf "${RED}✖ Unsupported architecture: %s for Linux.${NC}\n" "$ARCH"
                exit 1
                ;;
        esac
        ;;
    darwin)
        case "$ARCH" in
            x86_64)
                ASSET_ARCH="macos-x86_64"
                ;;
            arm64|aarch64)
                ASSET_ARCH="macos-arm64"
                ;;
            *)
                printf "${RED}✖ Unsupported architecture: %s for macOS.${NC}\n" "$ARCH"
                exit 1
                ;;
        esac
        ;;
    *)
        printf "${RED}✖ Unsupported OS: %s. Use install.ps1 for Windows.${NC}\n" "$OS"
        exit 1
        ;;
esac

ASSET_NAME="${APP_NAME}-${ASSET_ARCH}.tar.gz"

if [ "$VERSION" = "latest" ]; then
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"
else
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET_NAME}"
fi

# 2. Check for yt-dlp & ffmpeg (runtime dependencies)
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

# 4. Download and Install Pre-compiled Binary
printf "${CYAN}● Downloading %s from GitHub Releases...${NC}\n" "$ASSET_NAME"
printf "  URL: %s\n" "$DOWNLOAD_URL"

TMP_DIR="$(mktemp -d -t anime-dl-XXXXXX)"
ARCHIVE_PATH="$TMP_DIR/$ASSET_NAME"

if curl -fsSL "$DOWNLOAD_URL" -o "$ARCHIVE_PATH"; then
    tar -xzf "$ARCHIVE_PATH" -C "$TMP_DIR"
    cp "$TMP_DIR/$APP_NAME" "$INSTALL_DIR/$APP_NAME"
    chmod +x "$INSTALL_DIR/$APP_NAME"
    rm -rf "$TMP_DIR"
    printf "${GREEN}✔ Installed %s to %s/%s${NC}\n" "$APP_NAME" "$INSTALL_DIR" "$APP_NAME"
else
    rm -rf "$TMP_DIR"
    printf "${RED}✖ Failed to download release asset from GitHub.${NC}\n"
    printf "  Ensure a GitHub release exists at https://github.com/%s/releases\n" "$REPO"
    
    # Fallback to cargo build if cargo is available
    if command -v cargo >/dev/null 2>&1; then
        printf "\n${YELLOW}● Cargo detected. Attempting fallback via 'cargo install'...${NC}\n"
        cargo install --git "https://github.com/${REPO}.git"
    else
        exit 1
    fi
fi

# 5. Verify PATH
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
