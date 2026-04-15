use tauri::State;
use crate::core::searcher::BilibiliSearcher;
use crate::core::lyrics_client::{LyricsClient, fetch_subtitle};
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

    // Parallel fetch: NetEase LRC + B站 subtitle
    let lrc_future = lyrics_client.fetch_lyrics(&keyword);
    let sub_future = searcher.get_subtitle_url(&bvid);

    let (lrc_result, sub_result) = tokio::join!(lrc_future, sub_future);

    let mut lyrics_data = lrc_result.unwrap_or_else(|_| crate::core::lyrics_client::LyricsData {
        lyrics: vec![],
        karaoke: None,
        song: None,
        artist: None,
    });

    // Try to fetch and parse subtitle for karaoke effect
    if let Ok(Some(subtitle_url)) = sub_result {
        match fetch_subtitle(&subtitle_url).await {
            Ok(karaoke) if !karaoke.is_empty() => {
                lyrics_data.karaoke = Some(karaoke);
            }
            _ => {}
        }
    }

    Ok(lyrics_data)
}

#[tauri::command]
pub async fn fetch_cover_url(
    bvid: String,
    searcher: State<'_, BilibiliSearcher>,
) -> AppResult<String> {
    let song_info = searcher.get_view_info(&bvid).await?;

    Ok(song_info.cover_url)
}
