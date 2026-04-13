use tauri::State;
use crate::core::searcher::BilibiliSearcher;
use crate::core::lyrics_client::LyricsClient;
use crate::error::{AppError, AppResult};

#[tauri::command]
pub async fn fetch_lyrics(
    bvid: String,
    searcher: State<'_, BilibiliSearcher>,
    lyrics_client: State<'_, LyricsClient>,
) -> AppResult<crate::core::lyrics_client::LyricsData> {
    if !regex::Regex::new(r"^BV[a-zA-Z0-9]+$")
        .unwrap()
        .is_match(&bvid)
    {
        return Err(AppError::InvalidParams("无效的 BV 号格式".into()));
    }

    let song_info = searcher.get_view_info(&bvid).await?;

    let keyword = format!("{} {}", song_info.title, song_info.author);
    lyrics_client.fetch_lyrics(&keyword).await
}

#[tauri::command]
pub async fn fetch_cover_url(
    bvid: String,
    searcher: State<'_, BilibiliSearcher>,
) -> AppResult<String> {
    let song_info = searcher.get_view_info(&bvid).await?;

    Ok(song_info.cover_url)
}
