mod config;
mod error;
mod core;
mod commands;

use core::searcher::BilibiliSearcher;
use core::lyrics_client::LyricsClient;
use core::task_manager::TaskManager;
use core::ffmpeg_path::FfmpegPath;
use tauri::Manager;

pub fn run() {
    let searcher_for_cover = BilibiliSearcher::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
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

            // Initialize Bilibili cookies in background
            tauri::async_runtime::spawn(async {
                core::downloader::init_bili_client().await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search::search_bilibili,
            commands::search::get_related,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
