param(
    [switch]$Force
)

$ErrorActionPreference = "Stop"

$BinDir = Join-Path $PSScriptRoot ".." "src-tauri" "binaries"
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

    # Use .NET HttpClient for better progress display and TLS support
    $ProgressPreference = "SilentlyContinue"
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

    $httpClient = [System.Net.Http.HttpClient]::new()
    $response = $httpClient.GetAsync($DownloadUrl, [System.Net.Http.HttpCompletionOption]::ResponseHeadersRead).Result
    $response.EnsureSuccessStatusCode()

    $totalBytes = $response.Content.Headers.ContentLength
    $fileStream = [System.IO.File]::Create($ZipPath)
    $stream = $response.Content.ReadAsStreamAsync().Result
    $buffer = New-Object byte[] 81920
    $downloadedBytes = 0
    $lastUpdate = [DateTime]::UtcNow

    while ($true) {
        $read = $stream.ReadAsync($buffer, 0, $buffer.Length).Result
        if ($read -eq 0) { break }
        $fileStream.Write($buffer, 0, $read)
        $downloadedBytes += $read

        $now = [DateTime]::UtcNow
        if (($now - $lastUpdate).TotalMilliseconds -gt 500) {
            if ($totalBytes -gt 0) {
                $percent = [math]::Round($downloadedBytes / $totalBytes * 100)
                $downloadedMB = [math]::Round($downloadedBytes / 1MB, 1)
                $totalMB = [math]::Round($totalBytes / 1MB, 1)
                Write-Host "`r[download-ffmpeg] Downloading... $downloadedMB MB / $totalMB MB ($percent%)" -NoNewline -ForegroundColor Cyan
            }
            $lastUpdate = $now
        }
    }

    Write-Host ""
    $fileStream.Close()
    $httpClient.Dispose()

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
