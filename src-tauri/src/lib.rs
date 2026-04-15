mod config;
mod error;
mod core;
mod commands;

use core::searcher::BilibiliSearcher;
use core::lyrics_client::LyricsClient;
use core::task_manager::TaskManager;
use core::ffmpeg_path::FfmpegPath;
use commands::settings::SharedPlayerState;
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::menu::{MenuBuilder, MenuItemBuilder};

pub fn run() {
    let searcher_for_cover = BilibiliSearcher::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .register_asynchronous_uri_scheme_protocol("bili-cover", {
            let client = std::sync::Arc::new(reqwest::Client::new());
            let searcher = searcher_for_cover.clone();
            move |_ctx, request, responder| {
                let client = client.clone();
                let searcher = searcher.clone();
                let uri = request.uri().to_string();
                let bvid = uri
                    .replace("bili-cover://", "")
                    .replace("bili-cover:", "")
                    .split('?')
                    .next()
                    .unwrap_or("")
                    .to_string();

                tauri::async_runtime::spawn(async move {
                    if bvid.is_empty() {
                        responder.respond(
                            tauri::http::Response::builder()
                                .status(tauri::http::StatusCode::BAD_REQUEST)
                                .body(Vec::new())
                                .unwrap(),
                        );
                        return;
                    }

                    let cover_url = match searcher.get_view_info(&bvid).await {
                        Ok(info) => Some(info.cover_url),
                        Err(_) => None,
                    };

                    if let Some(url) = cover_url {
                        match client.get(&url).header("User-Agent", config::USER_AGENT).header("Referer", "https://www.bilibili.com").send().await {
                            Ok(resp) => {
                                let ct = resp
                                    .headers()
                                    .get("content-type")
                                    .and_then(|v| v.to_str().ok())
                                    .unwrap_or("image/jpeg")
                                    .to_string();

                                match resp.bytes().await {
                                    Ok(body) => {
                                        responder.respond(
                                            tauri::http::Response::builder()
                                                .header("content-type", &ct)
                                                .header("cache-control", "public, max-age=3600")
                                                .body(body.to_vec())
                                                .unwrap(),
                                        );
                                    }
                                    Err(_) => {
                                        responder.respond(
                                            tauri::http::Response::builder()
                                                .status(tauri::http::StatusCode::BAD_GATEWAY)
                                                .body(Vec::new())
                                                .unwrap(),
                                        );
                                    }
                                }
                            }
                            Err(_) => {
                                responder.respond(
                                    tauri::http::Response::builder()
                                        .status(tauri::http::StatusCode::BAD_GATEWAY)
                                        .body(Vec::new())
                                        .unwrap(),
                                );
                            }
                        }
                    } else {
                        responder.respond(
                            tauri::http::Response::builder()
                                .status(tauri::http::StatusCode::NOT_FOUND)
                                .body(Vec::new())
                                .unwrap(),
                        );
                    }
                });
            }
        })
        .setup(|app| {
            let searcher = BilibiliSearcher::new();
            let lyrics_client = LyricsClient::new();
            let task_manager = TaskManager::new(app.handle().clone());
            let ffmpeg_path = FfmpegPath(FfmpegPath::resolve(app.handle()));

            app.manage(searcher);
            app.manage(lyrics_client);
            app.manage(task_manager);
            app.manage(ffmpeg_path);
            app.manage(Mutex::new(SharedPlayerState::default()));

            // ── Restore window geometry ──
            {
                let window = app.get_webview_window("main").unwrap();
                let settings_path = app.path()
                    .app_data_dir()
                    .unwrap_or_else(|_| config::get_default_output_dir().parent().unwrap().to_path_buf())
                    .join("settings.json");

                if settings_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&settings_path) {
                        if let Ok(settings) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(geo) = settings.get("windowGeometry") {
                                let x = geo.get("x").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                                let y = geo.get("y").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                                let w = geo.get("width").and_then(|v| v.as_u64()).unwrap_or(1100) as u32;
                                let h = geo.get("height").and_then(|v| v.as_u64()).unwrap_or(750) as u32;
                                let maximized = geo.get("maximized").and_then(|v| v.as_bool()).unwrap_or(false);

                                // Clamp to minimum size
                                let w = w.max(800);
                                let h = h.max(600);

                                // Check if position is on screen
                                let monitors = window.available_monitors().unwrap_or_default();
                                let mut on_screen = monitors.is_empty();
                                for mon in &monitors {
                                    let mon_size = mon.size();
                                    let mon_pos = mon.position();
                                    if x >= mon_pos.x && y >= mon_pos.y
                                        && x < mon_pos.x + mon_size.width as i32
                                        && y < mon_pos.y + mon_size.height as i32 {
                                        on_screen = true;
                                        break;
                                    }
                                }

                                if on_screen {
                                    let _ = window.set_size(tauri::PhysicalSize::new(w, h));
                                    let _ = window.set_position(tauri::PhysicalPosition::new(x, y));
                                }

                                if maximized {
                                    let _ = window.maximize();
                                }
                            }
                        }
                    }
                }
            }

            // Initialize Bilibili cookies in background
            tauri::async_runtime::spawn(async {
                core::downloader::init_bili_client().await;
            });

            // ── System tray ──
            let show_item = MenuItemBuilder::with_id("show", "显示窗口").build(app)?;
            let play_pause_item = MenuItemBuilder::with_id("play_pause", "播放 / 暂停").build(app)?;
            let next_item = MenuItemBuilder::with_id("next", "下一首").build(app)?;
            let prev_item = MenuItemBuilder::with_id("prev", "上一首").build(app)?;
            let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show_item)
                .separator()
                .item(&play_pause_item)
                .item(&prev_item)
                .item(&next_item)
                .separator()
                .item(&quit_item)
                .build()?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Bili Music")
                .icon(app.default_window_icon().unwrap().clone())
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        // Hide mini-player when showing main
                        if let Some(w) = app.get_webview_window("mini-player") {
                            let _ = w.hide();
                        }
                    }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        if let Some(w) = app.get_webview_window("mini-player") {
                            let _ = w.hide();
                        }
                    }
                    "play_pause" => {
                        let _ = app.emit("tray-play-pause", ());
                    }
                    "next" => {
                        let _ = app.emit("tray-next", ());
                    }
                    "prev" => {
                        let _ = app.emit("tray-prev", ());
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            // ── Close to tray ──
            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    // Check settings: if minimize_to_tray is false, allow close
                    let app_handle = window_clone.app_handle();
                    let settings_path = app_handle
                        .path()
                        .app_data_dir()
                        .unwrap_or_else(|_| config::get_default_output_dir().parent().unwrap().to_path_buf())
                        .join("settings.json");

                    let minimize_to_tray = if settings_path.exists() {
                        std::fs::read_to_string(&settings_path)
                            .ok()
                            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                            .and_then(|v| v.get("minimizeToTray").and_then(|v| v.as_bool()))
                            .unwrap_or(true)
                    } else {
                        true
                    };

                    if minimize_to_tray {
                        api.prevent_close();
                        let _ = window_clone.hide();
                        // Hide mini-player but keep desktop-lyrics visible
                        if let Some(w) = app_handle.get_webview_window("mini-player") {
                            let _ = w.hide();
                        }
                    }
                }
            });

            // ── Global shortcuts ──
            use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

            let gs = app.global_shortcut();
            if let Err(e) = gs.on_shortcuts(
                ["ctrl+shift+space", "ctrl+shift+left", "ctrl+shift+right"],
                move |app_handle, shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        match shortcut.to_string().as_str() {
                            "ctrl+shift+space" => {
                                let _ = app_handle.emit("global-play-pause", ());
                            }
                            "ctrl+shift+left" => {
                                let _ = app_handle.emit("global-prev", ());
                            }
                            "ctrl+shift+right" => {
                                let _ = app_handle.emit("global-next", ());
                            }
                            _ => {}
                        }
                    }
                },
            ) {
                eprintln!("[tray] Failed to register global shortcuts: {}", e);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search::search_bilibili,
            commands::search::get_related,
            commands::search::get_hot_ranking,
            commands::search::get_now_playing,
            commands::search::get_video_comments,
            commands::search::get_comment_replies,
            commands::search::get_danmaku,
            commands::search::check_login,
            commands::search::get_user_info,
            commands::search::get_popular,
            commands::download::start_download,
            commands::download::get_download_progress,
            commands::download::get_downloaded_file_path,
            commands::download::open_in_explorer,
            commands::download::get_audio_url,
            commands::download::stream_audio,
            commands::lyrics::fetch_lyrics,
            commands::lyrics::fetch_cover_url,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::pick_directory,
            commands::settings::check_tools,
            commands::settings::update_player_state,
            commands::settings::get_player_state,
            commands::settings::emit_to_main,
            commands::favorites_import::fetch_user_favorites_folders,
            commands::favorites_import::fetch_favorites_folder_videos,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
