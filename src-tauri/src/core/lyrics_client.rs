use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::config::*;
use crate::error::AppResult;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LyricLine {
    pub time: f64,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct LyricsData {
    pub lyrics: Vec<LyricLine>,
    pub song: Option<String>,
    pub artist: Option<String>,
}

pub struct LyricsClient {
    client: Client,
}

impl LyricsClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .default_headers(netease_headers())
            .build()
            .expect("Failed to build HTTP client");
        Self { client }
    }

    pub async fn fetch_lyrics(&self, keyword: &str) -> AppResult<LyricsData> {
        let songs = self.search_song(keyword).await?;

        if songs.is_empty() {
            return Ok(LyricsData {
                lyrics: vec![],
                song: None,
                artist: None,
            });
        }

        for (song_id, song_name, artist) in &songs {
            let lrc_text = self.get_lyric(song_id).await?;
            if !lrc_text.is_empty() {
                return Ok(LyricsData {
                    lyrics: parse_lrc(&lrc_text),
                    song: Some(song_name.clone()),
                    artist: Some(artist.clone()),
                });
            }
        }

        Ok(LyricsData {
            lyrics: vec![],
            song: None,
            artist: None,
        })
    }

    async fn search_song(&self, keyword: &str) -> AppResult<Vec<(i64, String, String)>> {
        let resp = self
            .client
            .get(NETEASE_SEARCH_URL)
            .query(&[("s", keyword), ("type", "1"), ("offset", "0"), ("limit", "5")])
            .timeout(std::time::Duration::from_secs(8))
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;
        let songs = data
            .get("result")
            .and_then(|r| r.get("songs"))
            .and_then(|s| s.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(songs
            .iter()
            .filter_map(|song| {
                let id = song.get("id")?.as_i64()?;
                let name = song.get("name")?.as_str()?.to_string();
                let artist = song
                    .get("artists")?
                    .as_array()?
                    .first()?
                    .get("name")?
                    .as_str()?
                    .to_string();
                Some((id, name, artist))
            })
            .collect())
    }

    async fn get_lyric(&self, song_id: &i64) -> AppResult<String> {
        let resp = self
            .client
            .get(NETEASE_LYRIC_URL)
            .query(&[("id", song_id.to_string()), ("lv", "1".to_string())])
            .timeout(std::time::Duration::from_secs(8))
            .send()
            .await?;

        let data: serde_json::Value = resp.json().await?;
        let lrc = data
            .get("lrc")
            .and_then(|l| l.get("lyric"))
            .and_then(|l| l.as_str())
            .unwrap_or("");
        Ok(lrc.to_string())
    }
}

pub fn parse_lrc(lrc_text: &str) -> Vec<LyricLine> {
    let pattern = Regex::new(r"\[(\d{2}):(\d{2})\.(\d{2,3})\](.*)").unwrap();
    let mut lines = Vec::new();

    for raw in lrc_text.lines() {
        if let Some(caps) = pattern.captures(raw.trim()) {
            let minutes: u64 = caps[1].parse().unwrap_or(0);
            let seconds: u64 = caps[2].parse().unwrap_or(0);
            let millis_raw = caps[3].trim_end_matches('0');
            let millis_raw = if millis_raw.is_empty() {
                "0"
            } else {
                millis_raw
            };
            let millis_str = format!(
                "{}{}",
                millis_raw,
                "0".repeat(3 - millis_raw.len().min(3))
            );
            let millis: u64 = millis_str.parse().unwrap_or(0);

            let time_sec = minutes as f64 * 60.0 + seconds as f64 + millis as f64 / 1000.0;
            let text = caps[4].trim().to_string();

            if !text.is_empty() {
                lines.push(LyricLine {
                    time: (time_sec * 1000.0).round() / 1000.0,
                    text,
                });
            }
        }
    }

    lines.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
    lines
}
