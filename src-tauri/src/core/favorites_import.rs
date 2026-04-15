use serde::{Deserialize, Serialize};

use crate::config::*;
use crate::core::searcher::{format_play_count, SearchResult};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoritesFolder {
    pub id: i64,
    pub title: String,
    pub cover: String,
    pub media_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoritesFolderListResponse {
    pub folders: Vec<FavoritesFolder>,
    pub uid: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaResourcePage {
    pub videos: Vec<SearchResult>,
    pub page: u32,
    pub total: i64,
    pub has_more: bool,
}

pub async fn fetch_medialist(uid: &str) -> AppResult<FavoritesFolderListResponse> {
    let client = reqwest::Client::builder()
        .cookie_provider(super::downloader::BILI_JAR.clone())
        .default_headers(search_headers())
        .build()?;

    let resp: reqwest::Response = client
        .get(BILIBILI_MEDIALIST_URL)
        .query(&[("vmid", uid)])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await?;

    let status = resp.status();
    let bytes = resp.bytes().await?;

    if !status.is_success() {
        let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(300)]);
        return Err(AppError::Network(format!(
            "API 返回状态码 {}, 响应预览: {}",
            status, preview
        )));
    }

    let data: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| {
        let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(200)]);
        AppError::Other(format!("JSON 解析错误: {} (响应前200字节: {})", e, preview))
    })?;

    if data.get("code").and_then(|c| c.as_i64()) != Some(0) {
        let msg = data
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("获取收藏夹列表失败");
        return Err(AppError::Search(msg.into()));
    }

    let list = data
        .get("data")
        .and_then(|d| d.as_array())
        .cloned()
        .unwrap_or_default();

    // API already handles access control, no need to filter by state
    let folders: Vec<FavoritesFolder> = list
        .iter()
        .map(|item| {
            // cover is an array of objects, take the first one's pic
            let cover = item
                .get("cover")
                .and_then(|c| c.as_array())
                .and_then(|arr| arr.first())
                .and_then(|c| c.get("pic"))
                .and_then(|p| p.as_str())
                .unwrap_or("")
                .to_string();

            FavoritesFolder {
                id: item
                    .get("media_id")
                    .and_then(|i| i.as_i64())
                    .unwrap_or(0),
                title: item
                    .get("name")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string(),
                cover,
                media_count: item
                    .get("cur_count")
                    .and_then(|m| m.as_i64())
                    .unwrap_or(0),
            }
        })
        .collect();

    if folders.is_empty() {
        return Err(AppError::Search(
            "该用户没有公开的收藏夹".into(),
        ));
    }

    Ok(FavoritesFolderListResponse {
        folders,
        uid: uid.to_string(),
    })
}

pub async fn fetch_medialist_resources(
    uid: &str,
    media_id: i64,
    page: u32,
) -> AppResult<MediaResourcePage> {
    let page_size: u32 = 20;
    let client = reqwest::Client::builder()
        .cookie_provider(super::downloader::BILI_JAR.clone())
        .default_headers(search_headers())
        .build()?;

    let resp: reqwest::Response = client
        .get(BILIBILI_MEDIALIST_RESOURCE_URL)
        .query(&[
            ("media_id", &media_id.to_string()),
            ("pn", &page.to_string()),
            ("ps", &page_size.to_string()),
            ("up_mid", &uid.to_string()),
            ("order", &"mtime".to_string()),
        ])
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await?;

    let status = resp.status();
    let bytes = resp.bytes().await?;

    if !status.is_success() {
        let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(300)]);
        return Err(AppError::Network(format!(
            "API 返回状态码 {}, 响应预览: {}",
            status, preview
        )));
    }

    let data: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| {
        let preview = String::from_utf8_lossy(&bytes[..bytes.len().min(200)]);
        AppError::Other(format!("JSON 解析错误: {} (响应前200字节: {})", e, preview))
    })?;

    if data.get("code").and_then(|c| c.as_i64()) != Some(0) {
        let msg = data
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("获取收藏夹内容失败");
        return Err(AppError::Search(msg.into()));
    }

    let info = data.get("data").and_then(|d| d.get("info"));
    let media_count = info
        .and_then(|i| i.get("media_count"))
        .and_then(|m| m.as_i64())
        .unwrap_or(0);

    let items = data
        .get("data")
        .and_then(|d| d.get("medias"))
        .and_then(|m| m.as_array())
        .cloned()
        .unwrap_or_default();

    let videos: Vec<SearchResult> = items
        .iter()
        .map(|item| {
            let title = item
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            // medialist API returns clean titles (no HTML tags)

            let mut cover_url = item
                .get("cover")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string();
            if cover_url.starts_with("//") {
                cover_url = format!("https:{}", cover_url);
            }

            let upper = item
                .get("upper")
                .and_then(|u| u.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("")
                .to_string();

            let duration_secs = item
                .get("duration")
                .and_then(|d| d.as_u64())
                .unwrap_or(0);
            let duration = format_duration(duration_secs);

            let play_count = item
                .get("cnt_info")
                .and_then(|c| c.get("play"))
                .and_then(|p| p.as_u64())
                .unwrap_or(0);

            SearchResult {
                bvid: item
                    .get("bvid")
                    .and_then(|b| b.as_str())
                    .unwrap_or("")
                    .into(),
                title,
                author: upper,
                duration,
                play_count,
                play_count_text: format_play_count(play_count),
                cover_url,
                description: item
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .into(),
            }
        })
        .collect();

    let total_pages = ((media_count as f64) / (page_size as f64)).ceil() as u32;
    let has_more = page < total_pages;

    Ok(MediaResourcePage {
        videos,
        page,
        total: media_count,
        has_more,
    })
}

fn format_duration(total_secs: u64) -> String {
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{:02}:{:02}", minutes, seconds)
}
