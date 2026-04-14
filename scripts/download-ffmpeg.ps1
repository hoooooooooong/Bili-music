param(
    [switch]$Force
)

$ErrorActionPreference = "Stop"

$BinDir = Join-Path (Join-Path (Join-Path $PSScriptRoot "..") "src-tauri") "binaries"
$FfmpegPath = Join-Path $BinDir "ffmpeg.exe"

# Skip if ffmpeg.exe already exists (unless --Force)
if (-not $Force -and (Test-Path $FfmpegPath)) {
    Write-Host "[download-ffmpeg] ffmpeg.exe already exists, skipping." -ForegroundColor Green
    exit 0
}

# BtbN's GPL build includes libmp3lame (needed for MP3 encoding)
$DownloadUrl = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
$ZipPath = Join-Path $env:TEMP "bili-music-ffmpeg.zip"
$ExtractDir = Join-Path $env:TEMP "bili-music-ffmpeg-extract"

# Cleanup temp files from previous runs
if (Test-Path $ZipPath) { Remove-Item $ZipPath -Force }
if (Test-Path $ExtractDir) { Remove-Item $ExtractDir -Recurse -Force }

try {
    Write-Host "[download-ffmpeg] Downloading ffmpeg (GPL build with libmp3lame)..." -ForegroundColor Cyan

    # Use .NET HttpWebRequest for broad compatibility (Windows PowerShell 5+)
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    $ProgressPreference = "SilentlyContinue"

    Write-Host "[download-ffmpeg] Downloading... (this may take a while)" -ForegroundColor Cyan
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -UseBasicParsing

    Write-Host "[download-ffmpeg] Extracting ffmpeg.exe..." -ForegroundColor Cyan
    Expand-Archive -Path $ZipPath -DestinationPath $ExtractDir -Force

    # Find ffmpeg.exe in the extracted directory
    $extracted = Get-ChildItem -Path $ExtractDir -Recurse -Filter "ffmpeg.exe" | Select-Object -First 1
    if (-not $extracted) {
        Write-Error "[download-ffmpeg] ffmpeg.exe not found in the downloaded archive."
        exit 1
    }

    # Copy to target location
    if (-not (Test-Path $BinDir)) {
        New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
    }
    Copy-Item $extracted.FullName -Destination $FfmpegPath -Force

    $sizeMB = [math]::Round((Get-Item $FfmpegPath).Length / 1MB, 1)
    Write-Host "[download-ffmpeg] Done. ffmpeg.exe ($sizeMB MB) saved to src-tauri/binaries/" -ForegroundColor Green
}
finally {
    # Cleanup temp files
    if (Test-Path $ZipPath) { Remove-Item $ZipPath -Force -ErrorAction SilentlyContinue }
    if (Test-Path $ExtractDir) { Remove-Item $ExtractDir -Recurse -Force -ErrorAction SilentlyContinue }
}
