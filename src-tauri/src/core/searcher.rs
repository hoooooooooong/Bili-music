use reqwest::Client;
use reqwest::cookie::CookieStore;
use std::collections::HashMap;
use std::sync::Arc;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::config::*;
use crate::error::{AppError, AppResult};

fn read_varint(buf: &[u8], pos: &mut usize) -> u64 {
    let mut val: u64 = 0;
    let mut shift = 0;
    while *pos < buf.len() {
        let b = buf[*pos];
        *pos += 1;
        val |= ((b & 0x7f) as u64) << shift;
        shift += 7;
        if b & 0x80 == 0 {
            break;
        }
    }
    val
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub bvid: String,
    pub title: String,
    pub author: String,
    pub duration: String,
    pub play_count: u64,
    pub play_count_text: String,
    pub cover_url: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub page: u32,
    pub total: u64,
    pub page_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentMember {
    pub name: String,
    pub avatar: String,
    pub level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub rpid: i64,
    pub message: String,
    pub like: u64,
    pub rcount: u64,
    pub member: CommentMember,
    pub ctime: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentResponse {
    pub comments: Vec<Comment>,
    pub is_end: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Danmaku {
    pub progress: f64,
    pub content: String,
    pub color: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DanmakuResponse {
    pub danmaku: Vec<Danmaku>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub mid: u64,
    pub uname: String,
    pub face: String,
}

#[derive(Clone)]
pub struct BilibiliSearcher {
    client: Client,
    aid_cache: Arc<std::sync::Mutex<HashMap<String, i64>>>,
    cid_cache: Arc<std::sync::Mutex<HashMap<String, i64>>>,
}

impl BilibiliSearcher {
    pub fn new() -> Self {
        let jar = super::downloader::BILI_JAR.clone();
        let client = Client::builder()
            .cookie_provider(jar.clone())
            .default_headers(search_headers())
            .build()
            .expect("Failed to build HTTP client");

        let init_client = client.clone();
        tauri::async_runtime::spawn(async move {
            let _ = init_client.get("https://www.bilibili.com").send().await;
        });

        Self {
            client,
            aid_cache: Arc::new(std::sync::Mutex::new(HashMap::new())),
            cid_cache: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    pub async fn search(&self, keyword: &str, page: u32) -> AppResult<SearchResponse> {
        let params = [("keyword", keyword), ("page", &page.to_string())];

        let resp = self
            .client
            .get(BILIBILI_SEARCH_URL)
            .query(&params)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;

        if data.get("code").and_then(|c| c.as_i64()) != Some(0) {
            let msg = data
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("搜索失败");
            return Err(AppError::Search(msg.into()));
        }

        let result_groups = data
            .get("data")
            .and_then(|d| d.get("result"))
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        let mut video_results = Vec::new();
        for group in &result_groups {
            if group.get("result_type").and_then(|t| t.as_str()) == Some("video") {
                if let Some(items) = group.get("data").and_then(|d| d.as_array()) {
                    video_results = items.clone();
                }
                break;
            }
        }

        let num_results = data
            .get("data")
            .and_then(|d| d.get("numResults"))
            .and_then(|n| n.as_u64())
            .unwrap_or_else(|| video_results.len() as u64);

        if video_results.is_empty() {
            return Ok(SearchResponse {
                results: vec![],
                page,
                total: num_results,
                page_size: SEARCH_PAGE_SIZE,
            });
        }

        let re_tag = Regex::new(r"<[^>]+>").unwrap();
        let results: Vec<SearchResult> = video_results
            .iter()
            .take(SEARCH_PAGE_SIZE)
            .map(|item| Self::parse_item(item, &re_tag))
            .collect();

        Ok(SearchResponse {
            results,
            page,
            total: num_results,
            page_size: SEARCH_PAGE_SIZE,
        })
    }

    pub async fn get_view_info(&self, bvid: &str) -> AppResult<SearchResult> {
        let params = [("bvid", bvid)];
        let resp = self
            .client
            .get(BILIBILI_VIEW_URL)
            .query(&params)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;
        if data.get("code").and_then(|c| c.as_i64()) != Some(0) {
            let msg = data
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("获取视频详情失败");
            return Err(AppError::Search(msg.into()));
        }

        let item = data.get("data").ok_or_else(|| AppError::Search("获取视频详情数据为空".into()))?;
        let re_tag = Regex::new(r"<[^>]+>").unwrap();
        Ok(Self::parse_view_item(item, &re_tag))
    }

    /// Get current online viewer count for a video via /x/player/online/total
    pub async fn get_now_playing(&self, bvid: &str) -> AppResult<u64> {
        let cid = self.get_cid(bvid).await?;

        let resp = self
            .client
            .get("https://api.bilibili.com/x/player/online/total")
            .query(&[("bvid", bvid), ("cid", &cid.to_string())])
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;

        let total = data
            .get("data")
            .and_then(|d| d.get("total"))
            .and_then(|t| t.as_str())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        Ok(total)
    }

    /// Check if user is logged in by verifying SESSDATA cookie
    pub fn check_login() -> bool {
        let url = "https://bilibili.com".parse::<url::Url>().unwrap();
        let cookies = super::downloader::BILI_JAR.cookies(&url);
        if let Some(header) = cookies {
            let s = header.to_str().unwrap_or("");
            s.contains("SESSDATA=") && !s.contains("SESSDATA=;")
        } else {
            false
        }
    }

    /// Get current logged-in user info via /x/web-interface/nav
    pub async fn get_user_info(&self) -> AppResult<UserInfo> {
        let resp = self
            .client
            .get("https://api.bilibili.com/x/web-interface/nav")
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;

        if data.get("code").and_then(|c| c.as_i64()) != Some(0) {
            return Err(AppError::Search("获取用户信息失败，请检查登录状态".into()));
        }

        let info = data.get("data").ok_or_else(|| AppError::Search("用户信息数据为空".into()))?;

        Ok(UserInfo {
            mid: info.get("mid").and_then(|m| m.as_u64()).unwrap_or(0),
            uname: info.get("uname").and_then(|u| u.as_str()).unwrap_or("").to_string(),
            face: info.get("face").and_then(|f| f.as_str()).unwrap_or("").to_string(),
        })
    }

    /// Get music region (rid=3) videos from bilibili
    pub async fn get_popular(&self, page: u32, page_size: u32) -> AppResult<SearchResponse> {
        let params = [
            ("rid", "3"),
            ("ps", &page_size.to_string()),
            ("pn", &page.to_string()),
        ];

        let resp = self
            .client
            .get(BILIBILI_REGION_DYNAMIC_URL)
            .query(&params)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;

        if data.get("code").and_then(|c| c.as_i64()) != Some(0) {
            let msg = data
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("获取音乐区视频失败");
            return Err(AppError::Search(msg.into()));
        }

        let list = data
            .get("data")
            .and_then(|d| d.get("archives"))
            .and_then(|l| l.as_array())
            .cloned()
            .unwrap_or_default();

        let page_info = data
            .get("data")
            .and_then(|d| d.get("page"));

        let total = page_info
            .and_then(|p| p.get("count"))
            .and_then(|t| t.as_u64())
            .unwrap_or_else(|| list.len() as u64);

        let re_tag = Regex::new(r"<[^>]+>").unwrap();
        let results: Vec<SearchResult> = list
            .iter()
            .take(page_size as usize)
            .map(|item| {
                let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("");
                let title = decode_html_entities(title);
                let title = re_tag.replace_all(&title, "").to_string();

                let mut cover_url = item
                    .get("pic")
                    .and_then(|p| p.as_str())
                    .unwrap_or("")
                    .to_string();
                if cover_url.starts_with("//") {
                    cover_url = format!("https:{}", cover_url);
                }

                let play_count = item
                    .get("stat")
                    .and_then(|s| s.get("view"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                let author = item
                    .get("owner")
                    .and_then(|o| o.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();

                let duration_str = item
                    .get("duration")
                    .and_then(|d| d.as_str())
                    .unwrap_or("");
                let duration = if let Ok(secs) = duration_str.parse::<u64>() {
                    format!("{:02}:{:02}", secs / 60, secs % 60)
                } else {
                    let parts: Vec<&str> = duration_str.split(':').collect();
                    if parts.len() == 2 {
                        if let (Ok(m), Ok(s)) = (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
                            format!("{:02}:{:02}", m, s)
                        } else {
                            duration_str.to_string()
                        }
                    } else {
                        duration_str.to_string()
                    }
                };

                let desc = item
                    .get("desc")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string();

                SearchResult {
                    bvid: item.get("bvid").and_then(|b| b.as_str()).unwrap_or("").into(),
                    title: title.trim().into(),
                    author,
                    duration,
                    play_count,
                    play_count_text: format_play_count(play_count),
                    cover_url,
                    description: desc,
                }
            })
            .collect();

        Ok(SearchResponse {
            results,
            page,
            total,
            page_size: page_size as usize,
        })
    }

    /// Get Bilibili music section (rid=3) hot ranking
    pub async fn get_hot_ranking(&self) -> AppResult<Vec<SearchResult>> {
        let resp = self
            .client
            .get(BILIBILI_RANKING_URL)
            .query(&[("rid", "3")])
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;

        if data.get("code").and_then(|c| c.as_i64()) != Some(0) {
            let msg = data
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("获取排行榜失败");
            return Err(AppError::Search(msg.into()));
        }

        let re_tag = Regex::new(r"<[^>]+>").unwrap();
        let list = data
            .get("data")
            .and_then(|d| d.get("list"))
            .and_then(|l| l.as_array())
            .cloned()
            .unwrap_or_default();

        let results: Vec<SearchResult> = list
            .iter()
            .take(20)
            .map(|item| {
                let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("");
                let title = decode_html_entities(title);
                let title = re_tag.replace_all(&title, "").to_string();

                let mut cover_url = item
                    .get("pic")
                    .and_then(|p| p.as_str())
                    .unwrap_or("")
                    .to_string();
                if cover_url.starts_with("//") {
                    cover_url = format!("https:{}", cover_url);
                }

                let play_count = item
                    .get("stat")
                    .and_then(|s| s.get("view"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                let author = item
                    .get("owner")
                    .and_then(|o| o.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();

                let duration_secs = item
                    .get("duration")
                    .and_then(|d| d.as_u64())
                    .unwrap_or(0);
                let duration = format!("{:02}:{:02}", duration_secs / 60, duration_secs % 60);

                let desc = item
                    .get("desc")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string();

                SearchResult {
                    bvid: item.get("bvid").and_then(|b| b.as_str()).unwrap_or("").into(),
                    title: title.trim().into(),
                    author,
                    duration,
                    play_count,
                    play_count_text: format_play_count(play_count),
                    cover_url,
                    description: desc,
                }
            })
            .collect();

        Ok(results)
    }

    async fn get_aid(&self, bvid: &str) -> AppResult<i64> {
        if let Some(&aid) = self.aid_cache.lock().unwrap().get(bvid) {
            return Ok(aid);
        }

        let resp = self
            .client
            .get(BILIBILI_VIEW_URL)
            .query(&[("bvid", bvid)])
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;
        let data: serde_json::Value = resp.json().await?;
        let aid = data
            .get("data")
            .and_then(|d| d.get("aid"))
            .and_then(|a| a.as_i64())
            .ok_or_else(|| AppError::Search("获取视频 aid 失败".into()))?;
        self.aid_cache.lock().unwrap().insert(bvid.to_string(), aid);
        Ok(aid)
    }

    /// Get subtitle URL for a video via /x/player/wbi/v2
    pub async fn get_subtitle_url(&self, bvid: &str) -> AppResult<Option<String>> {
        let cid = self.get_cid(bvid).await?;

        let resp = self
            .client
            .get(BILIBILI_PLAYER_URL)
            .query(&[("bvid", bvid), ("cid", &cid.to_string())])
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;

        let subtitles = data
            .get("data")
            .and_then(|d| d.get("subtitle"))
            .and_then(|s| s.get("subtitles"))
            .and_then(|s| s.as_array())
            .cloned()
            .unwrap_or_default();

        for sub in &subtitles {
            let url = sub.get("subtitle_url").and_then(|u| u.as_str()).unwrap_or("");
            if !url.is_empty() {
                let full_url = if url.starts_with("//") {
                    format!("https:{}", url)
                } else if url.starts_with("http") {
                    url.to_string()
                } else {
                    format!("https://{}", url)
                };
                return Ok(Some(full_url));
            }
        }

        Ok(None)
    }

    async fn get_cid(&self, bvid: &str) -> AppResult<i64> {
        if let Some(&cid) = self.cid_cache.lock().unwrap().get(bvid) {
            return Ok(cid);
        }

        let resp = self
            .client
            .get(BILIBILI_VIEW_URL)
            .query(&[("bvid", bvid)])
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;
        let data: serde_json::Value = resp.json().await?;

        let cid = data
            .get("data")
            .and_then(|d| d.get("cid"))
            .and_then(|c| c.as_i64())
            .or_else(|| {
                data.get("data")
                    .and_then(|d| d.get("pages"))
                    .and_then(|p| p.as_array())
                    .and_then(|pages| pages.first())
                    .and_then(|p| p.get("cid"))
                    .and_then(|c| c.as_i64())
            })
            .ok_or_else(|| AppError::Search("获取视频 cid 失败".into()))?;

        self.cid_cache.lock().unwrap().insert(bvid.to_string(), cid);
        Ok(cid)
    }

    pub async fn get_danmaku(&self, bvid: &str) -> AppResult<DanmakuResponse> {
        let cid = self.get_cid(bvid).await?;
        eprintln!("[danmaku] bvid={} cid={}", bvid, cid);

        // Use the new segment-based danmaku API (protobuf format)
        // Each segment covers ~6 minutes (360 seconds)
        let mut danmaku_list = Vec::new();

        for segment_index in 1..=30 {
            let resp = self
                .client
                .get(BILIBILI_DANMAKU_SEG_URL)
                .query(&[
                    ("oid", &cid.to_string()),
                    ("segment_index", &segment_index.to_string()),
                    ("type", &"1".to_string()),
                ])
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await?;

            if resp.status() != reqwest::StatusCode::OK {
                break;
            }

            let body = resp.bytes().await?;
            if body.is_empty() || body.len() < 10 {
                break;
            }

            // Parse protobuf: top-level is repeated field 1 (length-delimited messages)
            let mut pos = 0;
            while pos < body.len() {
                let tag = body[pos];
                let field_num = tag >> 3;
                let wire_type = tag & 0x7;
                pos += 1;

                if field_num != 1 || wire_type != 2 {
                    // Skip unknown fields
                    if wire_type == 0 {
                        while pos < body.len() && body[pos] & 0x80 != 0 { pos += 1; }
                        if pos < body.len() { pos += 1; }
                    } else if wire_type == 2 {
                        let len = read_varint(&body, &mut pos);
                        pos += len as usize;
                    } else {
                        break;
                    }
                    continue;
                }

                let msg_len = read_varint(&body, &mut pos) as usize;
                if pos + msg_len > body.len() { break; }
                let msg = &body[pos..pos + msg_len];
                pos += msg_len;

                // Parse danmaku message fields:
                // field 1: fixed32 (id)
                // field 2: varint (progress in milliseconds)
                // field 3: varint (mode: 1=scroll, 4=top, 5=bottom)
                // field 4: varint (font size)
                // field 5: varint (color as uint32)
                // field 7: string (content)
                let mut progress_ms: f64 = 0.0;
                let mut mode: u32 = 1;
                let mut color_int: u32 = 16777215;
                let mut content = String::new();

                let mut mpos = 0;
                while mpos < msg.len() {
                    let mtag = msg[mpos];
                    let mfield = mtag >> 3;
                    let mwtype = mtag & 0x7;
                    mpos += 1;

                    match (mfield, mwtype) {
                        (1, 5) => { mpos += 4; } // fixed32 id
                        (2, 0) => { progress_ms = read_varint(msg, &mut mpos) as f64; }
                        (3, 0) => { mode = read_varint(msg, &mut mpos) as u32; }
                        (4, 0) => { let _ = read_varint(msg, &mut mpos); } // font size
                        (5, 0) => { color_int = read_varint(msg, &mut mpos) as u32; }
                        (7, 2) => {
                            let slen = read_varint(msg, &mut mpos) as usize;
                            if mpos + slen <= msg.len() {
                                content = String::from_utf8_lossy(&msg[mpos..mpos + slen]).to_string();
                                mpos += slen;
                            }
                        }
                        (8, 0) => { let _ = read_varint(msg, &mut mpos); } // timestamp
                        (9, 0) => { let _ = read_varint(msg, &mut mpos); } // pool type
                        _ => {
                            // Skip unknown field
                            if mwtype == 0 {
                                while mpos < msg.len() && msg[mpos] & 0x80 != 0 { mpos += 1; }
                                if mpos < msg.len() { mpos += 1; }
                            } else if mwtype == 2 {
                                let slen = read_varint(msg, &mut mpos) as usize;
                                mpos += slen;
                            } else if mwtype == 5 {
                                mpos += 4;
                            } else {
                                break;
                            }
                        }
                    }
                }

                let content = content.trim().to_string();
                if content.is_empty() {
                    continue;
                }

                // Only include scrolling danmaku (mode 1-3)
                if mode > 3 {
                    continue;
                }

                danmaku_list.push(Danmaku {
                    progress: progress_ms / 1000.0,
                    content,
                    color: format!("#{:06x}", color_int),
                });
            }
        }

        eprintln!("[danmaku] total parsed {} items", danmaku_list.len());

        danmaku_list.sort_by(|a, b| a.progress.partial_cmp(&b.progress).unwrap_or(std::cmp::Ordering::Equal));

        Ok(DanmakuResponse { danmaku: danmaku_list })
    }

    pub async fn get_comments(&self, bvid: &str, page: u32) -> AppResult<CommentResponse> {
        let aid = self.get_aid(bvid).await?;

        let params = [
            ("type", "1"),
            ("oid", &aid.to_string()),
            ("pn", &page.to_string()),
            ("ps", "20"),
            ("sort", "1"),
        ];

        let resp = self
            .client
            .get(BILIBILI_COMMENT_URL)
            .query(&params)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;

        if data.get("code").and_then(|c| c.as_i64()) != Some(0) {
            let msg = data
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("获取评论失败");
            return Err(AppError::Search(msg.into()));
        }

        let replies = data
            .get("data")
            .and_then(|d| d.get("replies"))
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        // cursor.is_end may be missing (empty cursor {}), fall back to count check
        let is_end = data
            .get("data")
            .and_then(|d| d.get("cursor"))
            .and_then(|c| c.get("is_end"))
            .and_then(|e| e.as_bool())
            .unwrap_or_else(|| replies.len() < 20);

        let comments: Vec<Comment> = replies.iter().map(|r| Self::parse_comment(r)).collect();

        Ok(CommentResponse { comments, is_end })
    }

    pub async fn get_replies(&self, bvid: &str, root: i64, page: u32) -> AppResult<CommentResponse> {
        let aid = self.get_aid(bvid).await?;

        let params = [
            ("type", "1"),
            ("oid", &aid.to_string()),
            ("root", &root.to_string()),
            ("pn", &page.to_string()),
            ("ps", "10"),
            ("sort", "1"),
        ];

        let resp = self
            .client
            .get(BILIBILI_REPLY_URL)
            .query(&params)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;

        if data.get("code").and_then(|c| c.as_i64()) != Some(0) {
            let msg = data
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("获取回复失败");
            return Err(AppError::Search(msg.into()));
        }

        let replies = data
            .get("data")
            .and_then(|d| d.get("replies"))
            .and_then(|r| r.as_array())
            .cloned()
            .unwrap_or_default();

        let is_end = data
            .get("data")
            .and_then(|d| d.get("cursor"))
            .and_then(|c| c.get("is_end"))
            .and_then(|e| e.as_bool())
            .unwrap_or_else(|| replies.len() < 10);

        let comments: Vec<Comment> = replies.iter().map(|r| Self::parse_comment(r)).collect();

        Ok(CommentResponse { comments, is_end })
    }

    fn parse_comment(item: &serde_json::Value) -> Comment {
        let member = item.get("member").unwrap_or(&serde_json::Value::Null);
        let name = member
            .get("uname")
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_string();
        let avatar = member
            .get("avatar")
            .and_then(|a| a.as_str())
            .unwrap_or("")
            .to_string();
        let level = member
            .get("level_info")
            .and_then(|l| l.get("current_level"))
            .and_then(|l| l.as_u64())
            .unwrap_or(0) as u32;

        Comment {
            rpid: item.get("rpid").and_then(|r| r.as_i64()).unwrap_or(0),
            message: item
                .get("content")
                .and_then(|c| c.get("message").or_else(|| c.get("msg")))
                .and_then(|m| m.as_str())
                .unwrap_or("")
                .to_string(),
            like: item.get("like").and_then(|l| l.as_u64()).unwrap_or(0),
            rcount: item.get("rcount").and_then(|r| r.as_u64()).unwrap_or(0),
            member: CommentMember { name, avatar, level },
            ctime: item.get("ctime").and_then(|t| t.as_i64()).unwrap_or(0),
        }
    }

    fn parse_view_item(item: &serde_json::Value, _re_tag: &Regex) -> SearchResult {
        let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("").to_string();
        
        let mut cover_url = item
            .get("pic")
            .and_then(|p| p.as_str())
            .unwrap_or("")
            .to_string();
        if cover_url.starts_with("//") {
            cover_url = format!("https:{}", cover_url);
        }

        let author = item.get("owner").and_then(|o| o.get("name")).and_then(|n| n.as_str()).unwrap_or("").to_string();
        let duration_secs = item.get("duration").and_then(|d| d.as_u64()).unwrap_or(0);
        let duration = format!("{:02}:{:02}", duration_secs / 60, duration_secs % 60);

        let play_count = item.get("stat").and_then(|s| s.get("view")).and_then(|v| v.as_u64()).unwrap_or(0);

        SearchResult {
            bvid: item.get("bvid").and_then(|b| b.as_str()).unwrap_or("").into(),
            title: title.trim().into(),
            author,
            duration,
            play_count,
            play_count_text: format_play_count(play_count),
            cover_url,
            description: item
                .get("desc")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .into(),
        }
    }

    fn parse_item(item: &serde_json::Value, re_tag: &Regex) -> SearchResult {
        let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("");
        let title = decode_html_entities(title);
        let title = re_tag.replace_all(&title, "").to_string();

        let mut cover_url = item
            .get("pic")
            .and_then(|p| p.as_str())
            .unwrap_or("")
            .to_string();
        if cover_url.starts_with("//") {
            cover_url = format!("https:{}", cover_url);
        }

        let play_count = item.get("play").and_then(|p| p.as_u64()).unwrap_or(0);

        let duration_str = item.get("duration").and_then(|d| d.as_str()).unwrap_or("");
        let duration = if let Ok(secs) = duration_str.parse::<u64>() {
            format!("{:02}:{:02}", secs / 60, secs % 60)
        } else {
            let parts: Vec<&str> = duration_str.split(':').collect();
            if parts.len() == 2 {
                if let (Ok(m), Ok(s)) = (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
                    format!("{:02}:{:02}", m, s)
                } else {
                    duration_str.to_string()
                }
            } else {
                duration_str.to_string()
            }
        };

        SearchResult {
            bvid: item.get("bvid").and_then(|b| b.as_str()).unwrap_or("").into(),
            title: title.trim().into(),
            author: item.get("author").and_then(|a| a.as_str()).unwrap_or("").into(),
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
    }
}

pub fn format_play_count(count: u64) -> String {
    if count >= 100_000_000 {
        format!("{:.1}亿", count as f64 / 100_000_000.0)
    } else if count >= 10_000 {
        format!("{}万", count / 10_000)
    } else {
        count.to_string()
    }
}

fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}
