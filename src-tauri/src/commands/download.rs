use std::path::PathBuf;
use tauri::State;
use crate::core::downloader::{BilibiliDownloader, download_from_url};
use crate::core::converter::AudioConverter;
use crate::core::ffmpeg_path::FfmpegPath;
use crate::core::searcher::BilibiliSearcher;
use crate::core::task_manager::TaskManager;
use crate::config::*;
use crate::error::{AppError, AppResult};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DownloadOptions {
    pub output_dir: Option<String>,
    pub format: Option<String>,
    pub quality: Option<String>,
}

#[tauri::command]
pub async fn start_download(
    bvid: String,
    state: State<'_, TaskManager>,
    searcher: State<'_, BilibiliSearcher>,
    ffmpeg_path: State<'_, FfmpegPath>,
    options: Option<DownloadOptions>,
) -> AppResult<String> {
    if !regex::Regex::new(r"^BV[a-zA-Z0-9]+$")
        .unwrap()
        .is_match(&bvid)
    {
        return Err(AppError::InvalidParams("无效的 BV 号格式".into()));
    }

    let task = state.create(&bvid).await;
    let task_id = task.task_id.clone();
    let task_id_for_return = task_id.clone();
    let bvid_clone = bvid.clone();

    // We need to get handles that outlive this function
    let state_inner = state.inner().clone();
    let searcher_inner = searcher.inner().clone();
    let ffmpeg = ffmpeg_path.0.clone();

    tokio::spawn(async move {
        let temp_dir = get_temp_dir();
        let options_for_dir = options.clone();
        let output_dir = options_for_dir
            .and_then(|o| o.output_dir)
            .map(PathBuf::from)
            .unwrap_or_else(get_default_output_dir);

        let downloader = BilibiliDownloader;
        let state_for_callback = state_inner.clone();
        let task_id_for_callback = task_id.clone();

        let downloaded_file =
            match downloader
                .download(&bvid_clone, move |downloaded, total| {
                    let state = state_for_callback.clone();
                    let tid = task_id_for_callback.clone();
                    tokio::spawn(async move {
                        let progress = if total > 0 {
                            (downloaded as f64 / total as f64) * 100.0
                        } else {
                            0.0
                        };
                        state
                            .update(&tid, |t| {
                                t.status = "downloading".into();
                                t.progress = progress;
                                t.downloaded_bytes = downloaded;
                                t.total_bytes = total;
                            })
                            .await;
                    });
                })
                .await
            {
                Ok(f) => f,
                Err(e) => {
                    state_inner
                        .update(&task_id, |t| {
                            t.status = "error".into();
                            t.error_message = Some(e.to_string());
                        })
                        .await;
                    return;
                }
            };

        // Search for video info
        let video_info = searcher_inner
            .search(&bvid_clone, 1, None)
            .await
            .ok()
            .and_then(|resp| resp.results.into_iter().find(|r| r.bvid == bvid_clone));

        // Download cover
        let cover_path = if let Some(ref info) = video_info {
            if !info.cover_url.is_empty() {
                let client = reqwest::Client::new();
                if let Ok(resp) = client
                    .get(&info.cover_url)
                    .headers(search_headers())
                    .timeout(std::time::Duration::from_secs(15))
                    .send()
                    .await
                {
                    if resp.status().is_success() {
                        let ct = resp
                            .headers()
                            .get("content-type")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("image/jpeg");
                        let ext = if ct.contains("png") {
                            ".png"
                        } else if ct.contains("webp") {
                            ".webp"
                        } else {
                            ".jpg"
                        };
                        let cover_file = temp_dir.join(format!("{}_cover{}", bvid_clone, ext));
                        if let Ok(bytes) = resp.bytes().await {
                            if tokio::fs::write(&cover_file, &bytes).await.is_ok() {
                                Some(cover_file)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        state_inner
            .update(&task_id, |t| {
                t.status = "converting".into();
                t.progress = 100.0;
            })
            .await;

        let raw_name = video_info
            .as_ref()
            .map(|i| format!("{} - {}", i.title, i.author))
            .unwrap_or_else(|| bvid_clone.clone());
        let safe_name = sanitize_filename(&raw_name);
        let format = options
            .as_ref()
            .and_then(|o| o.format.as_deref())
            .unwrap_or("mp3");
        let quality = options
            .as_ref()
            .and_then(|o| o.quality.as_deref())
            .unwrap_or("high");
        let output_path = output_dir.join(format!("{}.{}", safe_name, format));

        match AudioConverter::convert(
            &ffmpeg,
            &downloaded_file,
            &output_path,
            format,
            quality,
            cover_path.as_deref(),
            video_info.as_ref().map(|i| i.title.as_str()),
            video_info.as_ref().map(|i| i.author.as_str()),
        )
        .await
        {
            Ok(result_path) => {
                state_inner
                    .update(&task_id, |t| {
                        t.status = "done".into();
                        t.file_path = Some(result_path.to_str().unwrap().to_string());
                        t.file_name = Some(format!("{}.{}", safe_name, format));
                    })
                    .await;
            }
            Err(e) => {
                state_inner
                    .update(&task_id, |t| {
                        t.status = "error".into();
                        t.error_message = Some(e.to_string());
                    })
                    .await;
            }
        }

        // Cleanup
        let _ = tokio::fs::remove_file(&downloaded_file).await;
        if let Some(cp) = cover_path {
            let _ = tokio::fs::remove_file(cp).await;
        }
    });

    Ok(task_id_for_return)
}

#[tauri::command]
pub async fn get_download_progress(
    task_id: String,
    state: State<'_, TaskManager>,
) -> AppResult<crate::core::task_manager::TaskProgress> {
    let task = state
        .get(&task_id)
        .await
        .ok_or_else(|| AppError::TaskNotFound(format!("任务不存在: {}", task_id)))?;

    Ok(crate::core::task_manager::TaskProgress {
        task_id: task.task_id,
        bvid: task.bvid,
        status: task.status,
        progress: task.progress,
        downloaded_bytes: task.downloaded_bytes,
        total_bytes: task.total_bytes,
        downloaded_text: format_bytes(task.downloaded_bytes),
        total_text: format_bytes(task.total_bytes),
        file_path: task.file_path,
        file_name: task.file_name,
        error_message: task.error_message,
    })
}

fn format_bytes(num: u64) -> String {
    if num < 1024 {
        format!("{} B", num)
    } else if num < 1024 * 1024 {
        format!("{:.1} KB", num as f64 / 1024.0)
    } else {
        format!("{:.1} MB", num as f64 / (1024.0 * 1024.0))
    }
}

#[tauri::command]
pub async fn get_downloaded_file_path(
    task_id: String,
    state: State<'_, TaskManager>,
) -> AppResult<String> {
    let task = state
        .get(&task_id)
        .await
        .ok_or_else(|| AppError::TaskNotFound(format!("任务不存在: {}", task_id)))?;

    if task.status != "done" {
        return Err(AppError::FileNotReady("文件尚未生成".into()));
    }

    task.file_path
        .ok_or_else(|| AppError::FileNotReady("文件路径不存在".into()))
}

#[tauri::command]
pub async fn open_in_explorer(path: String) -> AppResult<()> {
    let p = std::path::Path::new(&path);
    let dir = p.parent().unwrap_or(p);
    opener::open(dir).map_err(|e| AppError::Other(format!("打开目录失败: {}", e)))?;
    Ok(())
}

#[tauri::command]
pub async fn get_audio_url(
    bvid: String,
) -> AppResult<crate::core::downloader::AudioUrlInfo> {
    BilibiliDownloader::get_audio_url(&bvid).await
}

#[tauri::command]
pub async fn stream_audio(bvid: String) -> AppResult<String> {
    let cache_dir = get_temp_dir();
    tokio::fs::create_dir_all(&cache_dir).await?;

    // Check cache: if audio file already exists, return it directly
    for ext in &["m4a", "webm", "mp4", "flv", "opus", "mp3"] {
        let candidate = cache_dir.join(format!("{}_audio.{}", bvid, ext));
        if candidate.exists() {
            let metadata = tokio::fs::metadata(&candidate).await?;
            if metadata.len() > 0 {
                return Ok(candidate.to_str().unwrap().to_string());
            }
        }
    }

    // Get audio URL from API
    let audio_info = BilibiliDownloader::get_audio_url(&bvid).await?;
    let output_path = cache_dir.join(format!("{}_audio.{}", bvid, audio_info.ext));

    download_from_url(
        &audio_info.url,
        &audio_info.backup_urls,
        &output_path,
        &|_, _| {},
    )
    .await?;

    Ok(output_path.to_str().unwrap().to_string())
}
