use std::path::PathBuf;
use std::sync::{Mutex, LazyLock, Arc};
use std::collections::HashMap;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
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
pub static BILI_JAR: LazyLock<Arc<reqwest::cookie::Jar>> =
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AudioUrlInfo {
    pub url: String,
    pub backup_urls: Vec<String>,
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

        let audio_info = Self::get_audio_url(bvid).await?;
        let ext = &audio_info.ext;
        let output_path = temp_dir.join(format!("{}.{}", bvid, ext));

        // If file already exists (from a previous failed attempt), use it
        if output_path.exists() {
            let metadata = tokio::fs::metadata(&output_path).await?;
            if metadata.len() > 0 {
                progress_callback(metadata.len(), metadata.len());
                return Ok(output_path);
            }
        }

        progress_callback(0, 0);

        download_from_url(
            &audio_info.url,
            &audio_info.backup_urls,
            &output_path,
            &progress_callback,
        )
        .await?;

        Ok(output_path)
    }

    pub async fn get_audio_url(bvid: &str) -> AppResult<AudioUrlInfo> {
        // Try cache first
        if let Some(cached) = AUDIO_URL_CACHE.get(bvid) {
            return Ok(cached);
        }

        let info = Self::get_audio_url_from_api(bvid).await?;
        AUDIO_URL_CACHE.set(bvid, info.clone());
        Ok(info)
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

        // Collect base URL and backup URLs
        let mut urls: Vec<String> = Vec::new();
        if let Some(u) = audio.get("baseUrl").or_else(|| audio.get("base_url")).and_then(|u| u.as_str()) {
            urls.push(u.to_string());
        }
        if let Some(backup) = audio.get("backupUrl").or_else(|| audio.get("backup_url")).and_then(|u| u.as_array()) {
            for u in backup {
                if let Some(s) = u.as_str() {
                    if !urls.contains(&s.to_string()) {
                        urls.push(s.to_string());
                    }
                }
            }
        }

        if urls.is_empty() {
            return Err(AppError::Download("音频 URL 为空".into()));
        }

        let url = urls[0].clone();
        let backup_urls = urls[1..].to_vec();

        Ok(AudioUrlInfo {
            url,
            backup_urls,
            ext: "m4a".to_string(),
            filesize: 0,
        })
    }
}

/// Download a file from a URL with progress callback, falling back to backup URLs.
pub async fn download_from_url<F>(
    url: &str,
    backup_urls: &[String],
    output_path: &std::path::Path,
    progress_callback: &F,
) -> AppResult<()>
where
    F: Fn(u64, u64) + Send + Sync + 'static,
{
    let mut last_error = None;

    // Try primary URL first, then backups
    let all_urls: Vec<&str> = std::iter::once(url)
        .chain(backup_urls.iter().map(|s| s.as_str()))
        .collect();

    for current_url in all_urls {
        match download_single_url(current_url, output_path, progress_callback).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                eprintln!("[download_from_url] URL {} failed: {}, trying next...", current_url, e);
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| AppError::Download("所有下载 URL 均失败".into())))
}

async fn download_single_url<F>(
    url: &str,
    output_path: &std::path::Path,
    progress_callback: &F,
) -> AppResult<()>
where
    F: Fn(u64, u64) + Send + Sync + 'static,
{
    let client = &*BILI_CLIENT;
    let resp = client
        .get(url)
        .timeout(std::time::Duration::from_secs(300))
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(AppError::Download(format!(
            "下载请求失败: HTTP {}",
            resp.status()
        )));
    }

    let total_size = resp.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = tokio::fs::File::create(output_path).await?;

    let mut stream = resp.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        progress_callback(downloaded, total_size);
    }

    file.flush().await?;
    Ok(())
}
