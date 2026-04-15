use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::config::*;
use crate::error::{AppError, AppResult};

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

#[derive(Clone)]
pub struct BilibiliSearcher {
    client: Client,
    aid_cache: Arc<std::sync::Mutex<HashMap<String, i64>>>,
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
