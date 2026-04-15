use tauri::State;
use crate::core::searcher::BilibiliSearcher;
use crate::error::AppResult;

#[tauri::command]
pub async fn search_bilibili(
    keyword: String,
    page: Option<u32>,
    order: Option<String>,
    searcher: State<'_, BilibiliSearcher>,
) -> AppResult<crate::core::searcher::SearchResponse> {
    let page = page.unwrap_or(1);
    if keyword.trim().is_empty() {
        return Err(crate::error::AppError::InvalidParams(
            "搜索关键词不能为空".into(),
        ));
    }
    searcher.search(&keyword, page, order.as_deref()).await
}

#[tauri::command]
pub async fn get_related(
    bvid: String,
    _searcher: State<'_, BilibiliSearcher>,
) -> AppResult<Vec<crate::core::searcher::SearchResult>> {
    use crate::config::*;

    let client = reqwest::Client::builder()
        .default_headers(search_headers())
        .build()?;

    let resp = client
        .get(BILIBILI_RELATED_URL)
        .query(&[("bvid", &bvid)])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    let data: serde_json::Value = resp.json().await?;

    if data.get("code").and_then(|c| c.as_i64()) != Some(0) {
        let msg = data
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("获取推荐失败");
        return Err(crate::error::AppError::Search(msg.into()));
    }

    let re_tag = regex::Regex::new(r"<[^>]+>").unwrap();
    let items = data
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();

    let mut results = Vec::new();
    for v in items.iter().take(10) {
        let title = v.get("title").and_then(|t| t.as_str()).unwrap_or("");
        let title = title.replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'");
        let title = re_tag.replace_all(&title, "").to_string();

        let mut pic = v
            .get("pic")
            .and_then(|p| p.as_str())
            .unwrap_or("")
            .to_string();
        if pic.starts_with("//") {
            pic = format!("https:{}", pic);
        }

        let play_count = v
            .get("stat")
            .and_then(|s| s.get("view"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        results.push(crate::core::searcher::SearchResult {
            bvid: v.get("bvid").and_then(|b| b.as_str()).unwrap_or("").into(),
            title: title.trim().into(),
            author: v
                .get("owner")
                .and_then(|o| o.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .into(),
            duration: v.get("duration").and_then(|d| d.as_str()).unwrap_or("").into(),
            play_count,
            play_count_text: crate::core::searcher::format_play_count(play_count),
            cover_url: pic,
            description: String::new(),
            pubdate: 0,
        });
    }

    Ok(results)
}

#[tauri::command]
pub async fn get_hot_ranking(
    searcher: State<'_, BilibiliSearcher>,
) -> AppResult<Vec<crate::core::searcher::SearchResult>> {
    searcher.get_hot_ranking().await
}

#[tauri::command]
pub async fn get_now_playing(
    bvid: String,
    searcher: State<'_, BilibiliSearcher>,
) -> AppResult<u64> {
    searcher.get_now_playing(&bvid).await
}

#[tauri::command]
pub async fn get_video_comments(
    bvid: String,
    page: Option<u32>,
    searcher: State<'_, BilibiliSearcher>,
) -> AppResult<crate::core::searcher::CommentResponse> {
    let page = page.unwrap_or(1);
    if bvid.trim().is_empty() {
        return Err(crate::error::AppError::InvalidParams(
            "bvid 不能为空".into(),
        ));
    }
    searcher.get_comments(&bvid, page).await
}

#[tauri::command]
pub async fn get_comment_replies(
    bvid: String,
    root: i64,
    page: Option<u32>,
    searcher: State<'_, BilibiliSearcher>,
) -> AppResult<crate::core::searcher::CommentResponse> {
    let page = page.unwrap_or(1);
    if bvid.trim().is_empty() {
        return Err(crate::error::AppError::InvalidParams(
            "bvid 不能为空".into(),
        ));
    }
    searcher.get_replies(&bvid, root, page).await
}

#[tauri::command]
pub async fn check_login() -> bool {
    BilibiliSearcher::check_login()
}

#[tauri::command]
pub async fn get_popular(
    page: Option<u32>,
    page_size: Option<u32>,
    searcher: State<'_, BilibiliSearcher>,
) -> AppResult<crate::core::searcher::SearchResponse> {
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(20);
    searcher.get_popular(page, page_size).await
}

#[tauri::command]
pub async fn get_danmaku(
    bvid: String,
    searcher: State<'_, BilibiliSearcher>,
) -> AppResult<crate::core::searcher::DanmakuResponse> {
    if bvid.trim().is_empty() {
        return Err(crate::error::AppError::InvalidParams(
            "bvid 不能为空".into(),
        ));
    }
    searcher.get_danmaku(&bvid).await
}

#[tauri::command]
pub async fn get_user_info(
    searcher: State<'_, BilibiliSearcher>,
) -> AppResult<crate::core::searcher::UserInfo> {
    searcher.get_user_info().await
}
