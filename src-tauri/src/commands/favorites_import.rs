use crate::core::favorites_import;
use crate::error::{AppError, AppResult};

#[tauri::command]
pub async fn fetch_user_favorites_folders(
    uid: String,
) -> AppResult<favorites_import::FavoritesFolderListResponse> {
    let uid = uid.trim().to_string();
    if uid.is_empty() {
        return Err(AppError::InvalidParams("UID 不能为空".into()));
    }
    // Validate UID is numeric
    if uid.parse::<u64>().is_err() {
        return Err(AppError::InvalidParams("请输入有效的数字 UID".into()));
    }
    favorites_import::fetch_medialist(&uid).await
}

#[tauri::command]
pub async fn fetch_favorites_folder_videos(
    uid: String,
    media_id: i64,
    page: Option<u32>,
) -> AppResult<favorites_import::MediaResourcePage> {
    let page = page.unwrap_or(1);
    favorites_import::fetch_medialist_resources(&uid, media_id, page).await
}
