use std::path::PathBuf;
use tauri::Manager;

pub struct FfmpegPath(pub PathBuf);

impl FfmpegPath {
    /// Resolve ffmpeg path. In dev mode, reads from CARGO_MANIFEST_DIR/binaries/.
    /// In production, reads from the app's resource directory.
    pub fn resolve(app_handle: &tauri::AppHandle) -> PathBuf {
        if let Ok(resource_dir) = app_handle.path().resource_dir() {
            let path: PathBuf = resource_dir.join("binaries").join("ffmpeg.exe");
            if path.exists() {
                return path;
            }
        }

        // Dev fallback: look relative to CARGO_MANIFEST_DIR
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let path = PathBuf::from(manifest_dir)
                .join("binaries")
                .join("ffmpeg.exe");
            if path.exists() {
                return path;
            }
        }

        // Last resort: try PATH
        PathBuf::from("ffmpeg")
    }
}
