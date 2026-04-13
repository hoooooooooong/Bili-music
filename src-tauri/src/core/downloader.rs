use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Mutex, LazyLock, Arc};
use std::collections::HashMap;
use std::time::Instant;
use tokio::process::Command;
use serde::{Deserialize, Serialize};

use crate::config::*;
use crate::error::{AppError, AppResult};

struct AudioUrlCache {
    urls: Mutex<HashMap<String, (AudioUrlInfo, Instant)>>,
}

impl AudioUrlCache {
    fn new() -> Self {
        Self {
            urls: Mutex::new(HashMap::new()),
        }
    }

    fn get(&self, bvid: &str) -> Option<AudioUrlInfo> {
        let urls = self.urls.lock().unwrap();
        urls.get(bvid).and_then(|(info, inserted_at)| {
            if inserted_at.elapsed().as_secs() < 1800 {
                Some(info.clone())
            } else {
                None
            }
        })
    }

    fn set(&self, bvid: &str, info: AudioUrlInfo) {
        let mut urls = self.urls.lock().unwrap();
        urls.insert(bvid.to_string(), (info, Instant::now()));
    }
}

static AUDIO_URL_CACHE: LazyLock<AudioUrlCache> = LazyLock::new(AudioUrlCache::new);

/// Shared cookie jar for Bilibili API.
static BILI_JAR: LazyLock<Arc<reqwest::cookie::Jar>> =
    LazyLock::new(|| Arc::new(reqwest::cookie::Jar::default()));

/// Shared HTTP client with cookies for Bilibili API.
static BILI_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    let jar = BILI_JAR.clone();
    reqwest::Client::builder()
        .cookie_provider(jar)
        .default_headers(search_headers())
        .build()
        .expect("Failed to build HTTP client")
});

/// Ensure bilibili.com cookies are initialized (call once at startup).
pub async fn init_bili_client() {
    let client = &*BILI_CLIENT;
    let _ = client
        .get("https://www.bilibili.com")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;
}

/// Find yt-dlp executable path.
fn find_yt_dlp() -> Option<PathBuf> {
    // Try known paths first (fastest)
    if let Some(home) = dirs::home_dir() {
        let candidates = [
            home.join("AppData/Local/Python/pythoncore-3.14-64/Scripts/yt-dlp.exe"),
            home.join("AppData/Local/Python/pythoncore-3.13-64/Scripts/yt-dlp.exe"),
            home.join("AppData/Local/Python/pythoncore-3.12-64/Scripts/yt-dlp.exe"),
            home.join("AppData/Local/Programs/Python/Python314/Scripts/yt-dlp.exe"),
            home.join("AppData/Local/Programs/Python/Python313/Scripts/yt-dlp.exe"),
            home.join("AppData/Local/Programs/Python/Python312/Scripts/yt-dlp.exe"),
        ];
        for candidate in &candidates {
            if candidate.exists() {
                return Some(candidate.clone());
            }
        }
    }

    // Try PATH
    if let Ok(output) = std::process::Command::new("yt-dlp").arg("--version").output() {
        if output.status.success() {
            return Some(PathBuf::from("yt-dlp"));
        }
    }

    // Try python -m yt_dlp
    if let Ok(output) = std::process::Command::new("python")
        .args(["-m", "yt_dlp", "--version"])
        .output()
    {
        if output.status.success() {
            return Some(PathBuf::from("python"));
        }
    }

    None
}

/// Build yt-dlp command with proper args prefix.
fn yt_dlp_command() -> Command {
    let yt_dlp_path = find_yt_dlp();
    let is_python = yt_dlp_path.as_ref()
        .map(|p| p.to_str().unwrap_or("") == "python")
        .unwrap_or(false);

    let mut cmd = Command::new(yt_dlp_path.unwrap_or_else(|| PathBuf::from("yt-dlp")));
    if is_python {
        cmd.args(["-m", "yt_dlp"]);
    }
    cmd
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AudioUrlInfo {
    pub url: String,
    pub ext: String,
    pub filesize: u64,
}

pub struct BilibiliDownloader;

impl BilibiliDownloader {
    pub async fn download<F>(&self, bvid: &str, progress_callback: F) -> AppResult<PathBuf>
    where
        F: Fn(u64, u64) + Send + Sync + 'static,
    {
        let temp_dir = get_temp_dir();
        tokio::fs::create_dir_all(&temp_dir).await?;

        let output_template = temp_dir.join(format!("{}.%(ext)s", bvid));
        let output_template_str = output_template.to_str().unwrap().to_string();

        // Report initial progress
        progress_callback(0, 0);

        let output = yt_dlp_command()
            .args([
                "-f", "bestaudio/best",
                "--no-playlist",
                "--retries", "3",
                "--fragment-retries", "3",
                "--no-warnings",
                "--no-progress",
                "-o", &output_template_str,
                "--extractor-args", "bilibili:prefer_multi_flv_audio=False",
                &format!("{}{}", BILIBILI_VIDEO_URL, bvid),
            ])
            .output()
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    AppError::Other("未找到 yt-dlp，请安装并添加到 PATH".into())
                } else {
                    AppError::Download(format!("启动 yt-dlp 失败: {}", e))
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Download(format!("yt-dlp 下载失败: {}", stderr)));
        }

        // Report completion
        progress_callback(1, 1);

        let base = temp_dir.join(bvid);
        for ext in &["m4a", "webm", "mp4", "flv", "opus", "mp3"] {
            let candidate = base.with_extension(ext);
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        Err(AppError::Download("下载文件未找到".into()))
    }

    pub async fn get_audio_url(bvid: &str) -> AppResult<AudioUrlInfo> {
        // Try cache first
        if let Some(cached) = AUDIO_URL_CACHE.get(bvid) {
            return Ok(cached);
        }

        // Try Bilibili API first (fast)
        match Self::get_audio_url_from_api(bvid).await {
            Ok(info) => {
                AUDIO_URL_CACHE.set(bvid, info.clone());
                return Ok(info);
            }
            Err(e) => {
                eprintln!("[get_audio_url] API failed for {}: {}, trying yt-dlp", bvid, e);
            }
        }

        // Fallback to yt-dlp
        match Self::get_audio_url_from_ytdlp(bvid).await {
            Ok(info) => {
                AUDIO_URL_CACHE.set(bvid, info.clone());
                Ok(info)
            }
            Err(yt_err) => {
                eprintln!("[get_audio_url] yt-dlp also failed for {}: {}", bvid, yt_err);
                Err(yt_err)
            }
        }
    }

    async fn get_audio_url_from_api(bvid: &str) -> AppResult<AudioUrlInfo> {
        let client = &*BILI_CLIENT;

        // Get video info (aid, cid) from view API
        let view_resp = client
            .get(BILIBILI_VIEW_URL)
            .query(&[("bvid", bvid)])
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        let view_data: serde_json::Value = view_resp.json().await?;

        if view_data.get("code").and_then(|c| c.as_i64()) != Some(0) {
            let msg = view_data
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("获取视频信息失败");
            return Err(AppError::Download(msg.into()));
        }

        let video_data = view_data
            .get("data")
            .ok_or_else(|| AppError::Download("视频数据为空".into()))?;

        let aid = video_data
            .get("aid")
            .and_then(|a| a.as_u64())
            .unwrap_or(0);
        let cid = video_data
            .get("cid")
            .and_then(|c| c.as_u64())
            .unwrap_or(0);

        if cid == 0 {
            return Err(AppError::Download("获取 cid 失败".into()));
        }

        let resp = client
            .get(BILIBILI_PLAY_URL)
            .query(&[
                ("bvid", bvid),
                ("avid", &aid.to_string()),
                ("cid", &cid.to_string()),
                ("qn", "64"),
                ("fnval", "16"),
                ("fnver", "0"),
                ("fourk", "0"),
            ])
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;

        if data.get("code").and_then(|c| c.as_i64()) != Some(0) {
            let msg = data
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("获取播放地址失败");
            return Err(AppError::Download(msg.into()));
        }

        let dash = data
            .get("data")
            .and_then(|d| d.get("dash"))
            .ok_or_else(|| AppError::Download("未找到 DASH 数据".into()))?;

        let audio = dash
            .get("audio")
            .and_then(|a| a.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| AppError::Download("未找到音频流".into()))?;

        let url = audio
            .get("baseUrl")
            .or_else(|| audio.get("base_url"))
            .or_else(|| audio.get("backupUrl").and_then(|u| u.as_array()).and_then(|arr| arr.first()))
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();

        if url.is_empty() {
            return Err(AppError::Download("音频 URL 为空".into()));
        }

        Ok(AudioUrlInfo {
            url,
            ext: "m4a".to_string(),
            filesize: 0,
        })
    }

    async fn get_audio_url_from_ytdlp(bvid: &str) -> AppResult<AudioUrlInfo> {
        let output = yt_dlp_command()
            .args([
                "-f", "bestaudio/best",
                "--no-playlist",
                "--no-warnings",
                "--dump-json",
                "--header", &format!("User-Agent: {}", USER_AGENT),
                "--header", "Referer: https://www.bilibili.com",
                "--extractor-args", "bilibili:prefer_multi_flv_audio=False",
                &format!("{}{}", BILIBILI_VIDEO_URL, bvid),
            ])
            .output()
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    AppError::Other("未找到 yt-dlp，请安装: pip install yt-dlp".into())
                } else {
                    AppError::Download(format!("启动 yt-dlp 失败: {}", e))
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::Download(format!(
                "获取音频地址失败: {}",
                stderr
            )));
        }

        let info: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|e| AppError::Download(format!("解析 yt-dlp 输出失败: {}", e)))?;

        let mut url = info
            .get("url")
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();
        let mut ext = info
            .get("ext")
            .and_then(|e| e.as_str())
            .unwrap_or("m4a")
            .to_string();
        let mut filesize = info
            .get("filesize")
            .and_then(|f| f.as_u64())
            .or_else(|| info.get("filesize_approx").and_then(|f| f.as_u64()))
            .unwrap_or(0);

        if url.is_empty() {
            if let Some(formats) = info.get("formats").and_then(|f| f.as_array()) {
                for fmt in formats {
                    let vcodec = fmt
                        .get("vcodec")
                        .and_then(|v| v.as_str())
                        .unwrap_or("none");
                    if vcodec == "none" {
                        if let Some(u) = fmt.get("url").and_then(|u| u.as_str()) {
                            if !u.is_empty() {
                                url = u.to_string();
                                ext = fmt
                                    .get("ext")
                                    .and_then(|e| e.as_str())
                                    .unwrap_or(&ext)
                                    .to_string();
                                filesize = fmt
                                    .get("filesize")
                                    .and_then(|f| f.as_u64())
                                    .or_else(|| fmt.get("filesize_approx").and_then(|f| f.as_u64()))
                                    .unwrap_or(filesize);
                                break;
                            }
                        }
                    }
                }
            }
        }

        if url.is_empty() {
            return Err(AppError::Download("无法提取音频地址".into()));
        }

        Ok(AudioUrlInfo { url, ext, filesize })
    }
}

struct ProgressInfo {
    downloaded: u64,
    total: u64,
}

fn parse_yt_dlp_progress(line: &str) -> Option<ProgressInfo> {
    let line = line.trim();
    if !line.contains("[download]") && !line.contains("download:") {
        return None;
    }

    if let Some(rest) = line.strip_prefix("[download]") {
        let parts: Vec<&str> = rest.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            if *part == "/" && i >= 1 && i + 1 < parts.len() {
                if let (Some(dl), Some(total)) =
                    (parse_size_to_bytes(parts[i - 1]), parse_size_to_bytes(parts[i + 1]))
                {
                    return Some(ProgressInfo { downloaded: dl, total });
                }
            }
        }
    }

    None
}

fn parse_size_to_bytes(s: &str) -> Option<u64> {
    let s = s.trim();
    let (num_str, multiplier) = if s.ends_with("GiB") {
        (&s[..s.len() - 3], 1024u64 * 1024 * 1024)
    } else if s.ends_with("MiB") {
        (&s[..s.len() - 3], 1024u64 * 1024)
    } else if s.ends_with("KiB") {
        (&s[..s.len() - 3], 1024u64)
    } else if s.ends_with("GB") {
        (&s[..s.len() - 2], 1000u64 * 1000 * 1000)
    } else if s.ends_with("MB") {
        (&s[..s.len() - 2], 1000u64 * 1000)
    } else if s.ends_with("KB") {
        (&s[..s.len() - 2], 1000u64)
    } else if s.ends_with("B") && s.len() > 1 {
        (&s[..s.len() - 1], 1)
    } else {
        (s, 1)
    };

    let num: f64 = num_str.trim().parse().ok()?;
    Some((num * multiplier as f64) as u64)
}
