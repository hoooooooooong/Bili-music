use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("搜索失败: {0}")]
    Search(String),
    #[error("下载失败: {0}")]
    Download(String),
    #[error("转换失败: {0}")]
    Convert(String),
    #[error("歌词获取失败: {0}")]
    Lyrics(String),
    #[error("网络错误: {0}")]
    Network(String),
    #[error("参数错误: {0}")]
    InvalidParams(String),
    #[error("任务不存在: {0}")]
    TaskNotFound(String),
    #[error("文件未就绪: {0}")]
    FileNotReady(String),
    #[error("{0}")]
    Other(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AppError::Network("请求超时，请检查网络连接".into())
        } else if err.is_connect() {
            AppError::Network("网络连接失败，请检查网络".into())
        } else {
            AppError::Network(format!("网络请求失败: {}", err))
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Other(format!("IO 错误: {}", err))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Other(format!("JSON 解析错误: {}", err))
    }
}

impl From<tauri::Error> for AppError {
    fn from(err: tauri::Error) -> Self {
        AppError::Other(format!("Tauri 错误: {}", err))
    }
}

pub type AppResult<T> = Result<T, AppError>;
