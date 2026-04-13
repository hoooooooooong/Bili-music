use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgress {
    pub task_id: String,
    pub bvid: String,
    pub status: String,
    pub progress: f64,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub downloaded_text: String,
    pub total_text: String,
    pub file_path: Option<String>,
    pub file_name: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TaskInfo {
    pub task_id: String,
    pub bvid: String,
    pub status: String,
    pub progress: f64,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub file_path: Option<String>,
    pub file_name: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Clone)]
pub struct TaskManager {
    tasks: Arc<Mutex<HashMap<String, TaskInfo>>>,
    app_handle: tauri::AppHandle,
}

impl TaskManager {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            app_handle,
        }
    }

    pub async fn create(&self, bvid: &str) -> TaskInfo {
        let task = TaskInfo {
            task_id: uuid::Uuid::new_v4().to_string(),
            bvid: bvid.to_string(),
            status: "pending".into(),
            progress: 0.0,
            downloaded_bytes: 0,
            total_bytes: 0,
            file_path: None,
            file_name: None,
            error_message: None,
        };
        self.tasks.lock().await.insert(task.task_id.clone(), task.clone());
        task
    }

    pub async fn get(&self, task_id: &str) -> Option<TaskInfo> {
        self.tasks.lock().await.get(task_id).cloned()
    }

    pub async fn update(&self, task_id: &str, f: impl FnOnce(&mut TaskInfo)) {
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.get_mut(task_id) {
            f(task);
            let progress = TaskProgress {
                task_id: task.task_id.clone(),
                bvid: task.bvid.clone(),
                status: task.status.clone(),
                progress: task.progress,
                downloaded_bytes: task.downloaded_bytes,
                total_bytes: task.total_bytes,
                downloaded_text: format_bytes(task.downloaded_bytes),
                total_text: format_bytes(task.total_bytes),
                file_path: task.file_path.clone(),
                file_name: task.file_name.clone(),
                error_message: task.error_message.clone(),
            };
            let _ = self.app_handle.emit("download-progress", &progress);
        }
    }
}

fn format_bytes(num: u64) -> String {
    if num < 1024 {
        format!("{} B", num)
    } else if num < 1024 * 1024 {
        format!("{:.1} KB", num as f64 / 1024.0)
    } else {
        format!("{:.1} MB", num as f64 / (1024.0 * 1024.0))
    }
}
