pub const BILIBILI_SEARCH_URL: &str = "https://api.bilibili.com/x/web-interface/search/all/v2";
pub const BILIBILI_RELATED_URL: &str = "https://api.bilibili.com/x/web-interface/archive/related";
pub const BILIBILI_VIEW_URL: &str = "https://api.bilibili.com/x/web-interface/view";
pub const BILIBILI_PLAY_URL: &str = "https://api.bilibili.com/x/player/playurl";
pub const BILIBILI_VIDEO_URL: &str = "https://www.bilibili.com/video/";
pub const BILIBILI_RANKING_URL: &str = "https://api.bilibili.com/x/web-interface/ranking/v2";

pub const NETEASE_SEARCH_URL: &str = "https://music.163.com/api/search/get";
pub const NETEASE_LYRIC_URL: &str = "https://music.163.com/api/song/lyric";

pub const SEARCH_PAGE_SIZE: usize = 10;
pub const AUDIO_BITRATE: &str = "320k";
pub const AUDIO_SAMPLE_RATE: u32 = 44100;

pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

pub fn search_headers() -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", USER_AGENT.parse().unwrap());
    headers.insert("Referer", "https://www.bilibili.com".parse().unwrap());
    headers.insert("Origin", "https://www.bilibili.com".parse().unwrap());
    headers.insert("Accept", "application/json, text/plain, */*".parse().unwrap());
    headers.insert("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8".parse().unwrap());
    headers.insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
    headers.insert("Connection", "keep-alive".parse().unwrap());
    headers.insert("Sec-Fetch-Dest", "empty".parse().unwrap());
    headers.insert("Sec-Fetch-Mode", "cors".parse().unwrap());
    headers.insert("Sec-Fetch-Site", "same-origin".parse().unwrap());
    headers
}

pub fn download_headers() -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", USER_AGENT.parse().unwrap());
    headers.insert("Referer", "https://www.bilibili.com".parse().unwrap());
    headers
}

pub fn netease_headers() -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("User-Agent", USER_AGENT.parse().unwrap());
    headers.insert("Referer", "https://music.163.com".parse().unwrap());
    headers
}

pub fn get_temp_dir() -> std::path::PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("bili-music")
        .join("temp")
}

pub fn get_default_output_dir() -> std::path::PathBuf {
    dirs::audio_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from(".")))
        .join("BiliMusic")
}

pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}
