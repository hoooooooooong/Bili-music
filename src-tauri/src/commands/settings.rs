use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use crate::core::converter::AudioConverter;
use crate::config::get_default_output_dir;
use crate::error::AppResult;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub output_dir: String,
    pub theme: String,
    pub cache_size: u64,
    pub volume: f64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            output_dir: get_default_output_dir()
                .to_str()
                .unwrap_or("./output")
                .to_string(),
            theme: "dark".into(),
            cache_size: 500,
            volume: 1.0,
        }
    }
}

fn settings_file(app_handle: &AppHandle) -> std::path::PathBuf {
    app_handle
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| get_default_output_dir().parent().unwrap().to_path_buf())
        .join("settings.json")
}

#[tauri::command]
pub async fn get_settings(app_handle: AppHandle) -> AppResult<AppSettings> {
    let path = settings_file(&app_handle);
    if path.exists() {
        let content = tokio::fs::read_to_string(&path).await?;
        let settings: AppSettings = serde_json::from_str(&content)?;
        Ok(settings)
    } else {
        Ok(AppSettings::default())
    }
}

#[tauri::command]
pub async fn save_settings(app_handle: AppHandle, settings: AppSettings) -> AppResult<()> {
    let path = settings_file(&app_handle);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let content = serde_json::to_string_pretty(&settings)?;
    tokio::fs::write(&path, content).await?;
    Ok(())
}

#[tauri::command]
pub async fn pick_directory(app_handle: AppHandle) -> AppResult<Option<String>> {
    use tauri_plugin_dialog::DialogExt;
    let (sender, receiver) = tokio::sync::oneshot::channel();

    app_handle.dialog().file().pick_folder(move |path| {
        let _ = sender.send(path.map(|p| p.to_string()));
    });

    receiver
        .await
        .map_err(|_| crate::error::AppError::Other("对话框取消".into()))
}

#[tauri::command]
pub async fn check_tools() -> AppResult<serde_json::Value> {
    let ffmpeg = AudioConverter::check_ffmpeg().await;
    let ytdlp = AudioConverter::check_yt_dlp().await;
    Ok(serde_json::json!({
        "ffmpeg": ffmpeg,
        "ytdlp": ytdlp,
    }))
}
