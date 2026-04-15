use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};
use crate::core::converter::AudioConverter;
use crate::core::ffmpeg_path::FfmpegPath;
use crate::core::downloader::BILI_JAR;
use crate::config::get_default_output_dir;
use crate::error::AppResult;

/// Shared player state for cross-window communication (polling-based)
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SharedPlayerState {
    pub current_time: f64,
    pub duration: f64,
    pub is_playing: bool,
    pub current_song: Option<serde_json::Value>,
    pub cover_url: String,
    pub lyrics: Vec<serde_json::Value>,
    pub karaoke: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub maximized: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub output_dir: String,
    pub theme: String,
    pub cache_size: u64,
    pub volume: f64,
    pub download_format: String,
    pub download_quality: String,
    pub minimize_to_tray: bool,
    pub autostart_enabled: bool,
    #[serde(default = "default_accent_color")]
    pub accent_color: String,
    #[serde(default)]
    pub window_geometry: Option<WindowGeometry>,
    #[serde(default)]
    pub desktop_lyrics_enabled: bool,
    #[serde(default = "default_lyrics_font_size")]
    pub desktop_lyrics_font_size: u32,
    #[serde(default)]
    pub desktop_lyrics_locked: bool,
    #[serde(default)]
    pub sessdata: String,
}

fn default_accent_color() -> String { "#fb7299".into() }
fn default_lyrics_font_size() -> u32 { 32 }

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
            download_format: "mp3".into(),
            download_quality: "high".into(),
            minimize_to_tray: true,
            autostart_enabled: false,
            accent_color: default_accent_color(),
            window_geometry: None,
            desktop_lyrics_enabled: false,
            desktop_lyrics_font_size: default_lyrics_font_size(),
            desktop_lyrics_locked: false,
            sessdata: String::new(),
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
    let settings = if path.exists() {
        let content = tokio::fs::read_to_string(&path).await?;
        serde_json::from_str(&content)?
    } else {
        AppSettings::default()
    };

    // Inject SESSDATA into cookie jar on startup
    inject_sessdata(&settings.sessdata);

    Ok(settings)
}

#[tauri::command]
pub async fn save_settings(app_handle: AppHandle, settings: AppSettings) -> AppResult<()> {
    let path = settings_file(&app_handle);
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let content = serde_json::to_string_pretty(&settings)?;
    tokio::fs::write(&path, content).await?;

    // Inject SESSDATA into the shared cookie jar
    inject_sessdata(&settings.sessdata);

    Ok(())
}

fn inject_sessdata(sessdata: &str) {
    let url = url::Url::parse("https://bilibili.com").unwrap();
    if sessdata.is_empty() {
        BILI_JAR.add_cookie_str("SESSDATA=; Max-Age=0; Domain=bilibili.com", &url);
    } else {
        BILI_JAR.add_cookie_str(
            &format!("SESSDATA={}; Domain=bilibili.com", sessdata),
            &url,
        );
    }
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
pub async fn check_tools(ffmpeg_path: State<'_, FfmpegPath>) -> AppResult<serde_json::Value> {
    let ffmpeg = AudioConverter::check_ffmpeg(&ffmpeg_path.0).await;
    Ok(serde_json::json!({
        "ffmpeg": ffmpeg,
    }))
}

/// Update shared player state (called by main window)
#[tauri::command]
pub fn update_player_state(
    _app_handle: AppHandle,
    state: State<'_, Mutex<SharedPlayerState>>,
    new_state: SharedPlayerState,
) {
    let mut s = state.lock().unwrap();
    *s = new_state;
}

/// Get shared player state (called by mini-player / desktop-lyrics windows)
#[tauri::command]
pub fn get_player_state(
    _app_handle: AppHandle,
    state: State<'_, Mutex<SharedPlayerState>>,
) -> SharedPlayerState {
    let s = state.lock().unwrap();
    s.clone()
}

/// Emit an event to the main window (cross-window, reliable)
#[tauri::command]
pub fn emit_to_main(app_handle: AppHandle, event: String, payload: Option<serde_json::Value>) {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.emit(&event, payload);
    }
}

