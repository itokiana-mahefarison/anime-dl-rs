# ==============================================================================
# anime-dl-rs Installer for Windows (PowerShell)
# One-liner: irm https://raw.githubusercontent.com/itokiana/anime-dl-rs/master/install.ps1 | iex
# ==============================================================================

$ErrorActionPreference = "Stop"

$Repo = if ($env:REPO) { $env:REPO } else { "itokiana/anime-dl-rs" }
$AppName = "anime-dl-rs"
$Version = if ($env:VERSION) { $env:VERSION } else { "latest" }

Write-Host @"
               _                     _ _                
  __ _ _ _  (_)_ __  ___ ___ __| | |___ _ _ ___ 
 / _` | ' \ | | '  \/ -_)___/ _` | |___| '_(_-< 
 \__,_|_||_|_|_|_|_\___|   \__,_|_|   |_| /__/ 
"@ -ForegroundColor Cyan

Write-Host "`nInstalling $AppName for Windows...`n" -ForegroundColor White

# 1. Check for yt-dlp & ffmpeg (runtime dependencies)
if (-not (Get-Command yt-dlp -ErrorAction SilentlyContinue)) {
    Write-Host "⚠ Warning: 'yt-dlp' was not found in your PATH." -ForegroundColor Yellow
    Write-Host "  You can install it easily via winget:`n  winget install yt-dlp`n" -ForegroundColor DarkGray
}

if (-not (Get-Command ffmpeg -ErrorAction SilentlyContinue)) {
    Write-Host "⚠ Warning: 'ffmpeg' was not found in your PATH." -ForegroundColor Yellow
    Write-Host "  You can install it easily via winget:`n  winget install Gyan.FFmpeg`n" -ForegroundColor DarkGray
}

# 2. Determine Installation Target Directory
$InstallDir = "$env:USERPROFILE\.local\bin"
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# 3. Download and Install Pre-compiled Binary
$AssetName = "$AppName-windows-x86_64.zip"
$DownloadUrl = if ($Version -eq "latest") {
    "https://github.com/$Repo/releases/latest/download/$AssetName"
} else {
    "https://github.com/$Repo/releases/download/$Version/$AssetName"
}

Write-Host "● Downloading $AssetName from GitHub Releases..." -ForegroundColor Cyan
Write-Host "  URL: $DownloadUrl" -ForegroundColor DarkGray

$TmpZip = Join-Path $env:TEMP ("$AppName-" + [System.Guid]::NewGuid().ToString().Substring(0,8) + ".zip")
$TmpExtract = Join-Path $env:TEMP ("$AppName-extract-" + [System.Guid]::NewGuid().ToString().Substring(0,8))

try {
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $TmpZip -UseBasicParsing
    Expand-Archive -Path $TmpZip -DestinationPath $TmpExtract -Force
    
    $SourceBin = Join-Path $TmpExtract "$AppName.exe"
    if (-not (Test-Path $SourceBin)) {
        throw "Binary $AppName.exe not found in extracted archive."
    }
    
    $DestBin = Join-Path $InstallDir "$AppName.exe"
    Copy-Item -Path $SourceBin -Destination $DestBin -Force
    Write-Host "✔ Installed $AppName.exe to $DestBin" -ForegroundColor Green
}
catch {
    Write-Host "`n✖ Failed to download release asset: $_" -ForegroundColor Red
    Write-Host "  Ensure a GitHub release exists at https://github.com/$Repo/releases" -ForegroundColor Yellow

    # Fallback if Cargo is installed
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        Write-Host "`n● Cargo detected. Attempting fallback via 'cargo install'..." -ForegroundColor Cyan
        cargo install --git "https://github.com/$Repo.git"
    } else {
        exit 1
    }
}
finally {
    if (Test-Path $TmpZip) { Remove-Item -Force $TmpZip }
    if (Test-Path $TmpExtract) { Remove-Item -Recurse -Force $TmpExtract }
}

# 4. Ensure InstallDir is in User PATH
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
