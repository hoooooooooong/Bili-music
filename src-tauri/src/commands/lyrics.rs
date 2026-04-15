use tauri::State;
use crate::core::searcher::BilibiliSearcher;
use crate::core::lyrics_client::{LyricsClient, fetch_subtitle};
use crate::error::{AppError, AppResult};

/// 清洗B站视频标题，用于提高歌词搜索匹配率
fn clean_title_for_search(title: &str) -> String {
    let mut s = title.to_string();

    // 1. 去除 【...】和 [...]
    let bracket_re = regex::Regex::new(r"【.*?】|\[.*?\]").unwrap();
    s = bracket_re.replace_all(&s, "").to_string();

    // 2. 去除包含 cover/翻唱/remix/伴奏/mv/remastered/live/acoustic 的圆括号段
    let paren_re = regex::Regex::new(
        r"\((?i:cover|翻唱|remix|伴奏|mv|remastered|live|acoustic)[^)]*\)",
    )
    .unwrap();
    s = paren_re.replace_all(&s, "").to_string();

    // 3. 去除 `|` 分隔符及之后的内容
    if let Some(pos) = s.find('|') {
        s.truncate(pos);
    }

    // 4. 去除常见后缀关键词（仅作为独立词匹配）
    let suffixes = [
        r"(?i)\bMV\b",
        r"(?i)\bPV\b",
        r"(?i)\bOfficial Video\b",
        r"(?i)\bOfficial\b",
        r"(?i)\bLyric Video\b",
        r"(?i)\bLyrics\b",
        r"(?i)\bAudio\b",
        r"歌ってみた",
        r"(?i)\boff vocal\b",
        r"(?i)\binst\b",
        r"(?i)\binstrumental\b",
    ];
    for pat in &suffixes {
        let re = regex::Regex::new(pat).unwrap();
        s = re.replace_all(&s, "").to_string();
    }

    // 5. 去除 cover/翻唱/remix 等作为独立词
    let cover_re = regex::Regex::new(
        r"(?i)\b(翻唱版?|cover|remix)\b",
    )
    .unwrap();
    s = cover_re.replace_all(&s, "").to_string();

    // 6. 合并多余空格并 trim
    s = regex::Regex::new(r"\s+").unwrap().replace_all(&s.trim(), " ").to_string();

    s
}

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

    let clean_title = clean_title_for_search(&song_info.title);
    let keyword = format!("{} {}", clean_title, song_info.author);

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
