use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

use crate::config::*;
use crate::error::{AppError, AppResult};

pub struct AudioConverter;

impl AudioConverter {
    /// Convert audio file with given format and quality.
    ///
    /// Quality mapping:
    /// - MP3:  high=320k, medium=192k, low=128k
    /// - AAC:  high=320k, medium=192k, low=128k
    /// - FLAC: lossless (compression level 8/5/0)
    /// - WAV:  uncompressed PCM s16le
    pub async fn convert(
        ffmpeg_path: &Path,
        input_path: &Path,
        output_path: &Path,
        format: &str,
        quality: &str,
        cover_path: Option<&Path>,
        title: Option<&str>,
        artist: Option<&str>,
    ) -> AppResult<std::path::PathBuf> {
        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut cmd = Command::new(ffmpeg_path);
        cmd.arg("-i").arg(input_path);

        if let Some(cover) = cover_path {
            if cover.exists() {
                cmd.args(["-i", &cover.to_str().unwrap()])
                    .args(["-map", "0:a", "-map", "1:0"])
                    .args(["-c:v", "copy"])
                    .args(["-id3v2_version", "3"])
                    .args(["-metadata:s:v", "title=Album cover"])
                    .args(["-metadata:s:v", "comment=Cover (front)"]);
            }
        } else {
            cmd.arg("-vn");
        }

        // Apply codec + bitrate based on format + quality
        match format {
            "flac" => {
                let level = match quality {
                    "high" => "8",
                    "medium" => "5",
                    _ => "0",
                };
                cmd.args(["-c:a", "flac"])
                    .args(["-compression_level", level]);
            }
            "wav" => {
                cmd.args(["-c:a", "pcm_s16le"]);
            }
            "aac" => {
                let bitrate = quality_to_bitrate(quality);
                cmd.args(["-c:a", "aac"])
                    .args(["-ab", bitrate]);
            }
            _ => {
                // mp3
                let bitrate = quality_to_bitrate(quality);
                cmd.args(["-c:a", "libmp3lame"])
                    .args(["-ab", bitrate]);
            }
        }

        cmd.args(["-ar", &AUDIO_SAMPLE_RATE.to_string()])
            .args(["-ac", "2"]);

        if let Some(t) = title {
            cmd.args(["-metadata", &format!("title={}", t)]);
        }
        if let Some(a) = artist {
            cmd.args(["-metadata", &format!("artist={}", a)]);
        }

        cmd.args(["-y", &output_path.to_str().unwrap()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd.output().await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::Convert("未找到 ffmpeg".into())
            } else {
                AppError::Convert(format!("启动 ffmpeg 失败: {}", e))
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let msg = if stderr.len() > 500 {
                &stderr[stderr.len() - 500..]
            } else {
                &stderr
            };
            return Err(AppError::Convert(format!("ffmpeg 转换失败: {}", msg)));
        }

        let abs_path = output_path
            .canonicalize()
            .unwrap_or_else(|_| output_path.to_path_buf());
        Ok(abs_path)
    }

    /// Backward-compatible alias: convert to MP3 high quality.
    pub async fn to_mp3(
        ffmpeg_path: &Path,
        input_path: &Path,
        output_path: &Path,
        cover_path: Option<&Path>,
        title: Option<&str>,
        artist: Option<&str>,
    ) -> AppResult<std::path::PathBuf> {
        Self::convert(
            ffmpeg_path,
            input_path,
            output_path,
            "mp3",
            "high",
            cover_path,
            title,
            artist,
        )
        .await
    }

    pub async fn check_ffmpeg(ffmpeg_path: &Path) -> bool {
        Command::new(ffmpeg_path)
            .args(["-version"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

fn quality_to_bitrate(quality: &str) -> &'static str {
    match quality {
        "high" => "320k",
        "medium" => "192k",
        _ => "128k",
    }
}
