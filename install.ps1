# ==============================================================================
# anime-dl-rs Installer for Windows (PowerShell)
# One-liner: irm https://raw.githubusercontent.com/itokiana/anime-dl-rs/master/install.ps1 | iex
# ==============================================================================

$ErrorActionPreference = "Stop"

$RepoUrl = if ($env:REPO_URL) { $env:REPO_URL } else { "https://github.com/itokiana/anime-dl-rs.git" }
$AppName = "anime-dl-rs"

Write-Host @"
               _                     _ _                
  __ _ _ _  (_)_ __  ___ ___ __| | |___ _ _ ___ 
 / _` | ' \ | | '  \/ -_)___/ _` | |___| '_(_-< 
 \__,_|_||_|_|_|_|_\___|   \__,_|_|   |_| /__/ 
"@ -ForegroundColor Cyan

Write-Host "`nInstalling $AppName for Windows...`n" -ForegroundColor White

# 1. Check for Cargo / Rust
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "● Cargo/Rust is not detected on your system." -ForegroundColor Yellow
    Write-Host "Installing Rust via winget / rustup..." -ForegroundColor Cyan
    if (Get-Command winget -ErrorAction SilentlyContinue) {
        winget install Rustlang.Rustup -e --silent
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path","User") + ";" + [System.Environment]::GetEnvironmentVariable("Path","Machine")
    } else {
        Write-Host "Please install Rust from https://rustup.rs/ and re-run this script." -ForegroundColor Red
        exit 1
    }
}

# 2. Check for yt-dlp & ffmpeg
if (-not (Get-Command yt-dlp -ErrorAction SilentlyContinue)) {
    Write-Host "⚠ Warning: 'yt-dlp' was not found in your PATH." -ForegroundColor Yellow
    Write-Host "  You can install it easily via winget:`n  winget install yt-dlp`n" -ForegroundColor DarkGray
}

if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) {
    Write-Host "⚠ Warning: 'ffmpeg' was not found in your PATH." -ForegroundColor Yellow
    Write-Host "  You can install it easily via winget:`n  winget install Gyan.FFmpeg`n" -ForegroundColor DarkGray
}

# 3. Determine Installation Target Directory
$InstallDir = "$env:USERPROFILE\.local\bin"
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# 4. Clone and Build
$TmpDir = Join-Path $env:TEMP ("anime-dl-build-" + [System.Guid]::NewGuid().ToString().Substring(0,8))
Write-Host "● Cloning repository into temporary workspace..." -ForegroundColor Cyan
git clone --depth 1 $RepoUrl $TmpDir | Out-Null

Write-Host "● Compiling $AppName (release build with Cargo)..." -ForegroundColor Cyan
Push-Location $TmpDir
try {
    cargo build --release
} finally {
    Pop-Location
}

# 5. Move binary to installation path
$TargetBin = Join-Path $TmpDir "target\release\$AppName.exe"
if (-not (Test-Path $TargetBin)) {
    Write-Host "✖ Build failed: $TargetBin not found." -ForegroundColor Red
    Remove-Item -Recurse -Force $TmpDir
    exit 1
}

$DestBin = Join-Path $InstallDir "$AppName.exe"
Copy-Item -Path $TargetBin -Destination $DestBin -Force
Remove-Item -Recurse -Force $TmpDir

# 6. Ensure InstallDir is in User PATH
$UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "`n● Adding $InstallDir to your User PATH..." -ForegroundColor Cyan
    [System.Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    $env:Path += ";$InstallDir"
}

Write-Host "`n✔ $AppName successfully installed!`n" -ForegroundColor Green
Write-Host "Usage:" -ForegroundColor White
Write-Host "  $AppName                     # Launch modern interactive TUI" -ForegroundColor Cyan
Write-Host "  $AppName --help              # View CLI options" -ForegroundColor Cyan
Write-Host "  $AppName --url <ANIME_URL>   # Direct download`n" -ForegroundColor Cyan
