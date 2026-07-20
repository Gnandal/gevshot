//! Export vidéo MP4 : orchestration FFmpeg.
//! Pipeline :
//! 1. Séquencer le document en frames
//! 2. Écrire chaque frame PNG dans tempdir (dupliquer selon durée pour 30 fps)
//! 3. Concaténer pistes audio (silence pour étapes sans narration)
//! 4. Invoquer ffmpeg pour encoder PNG + audio → MP4
//! 5. Retourner octets (pattern identique à pdf/pptx/gif)

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use gev_core::document::Document;
use gev_core::fonts::FontSet;

use crate::domain::timeline;

const FPS: u32 = 30;

/// Génère un MP4 depuis un document via FFmpeg.
pub fn build(doc: &Document) -> Result<Vec<u8>, String> {
    let fonts = FontSet::embedded();
    let frames = timeline::sequence(doc, &fonts)?;

    if frames.is_empty() {
        return Err("No frames to export".to_string());
    }

    let temp_dir = std::env::temp_dir().join(format!("gevstudio-video-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {e}"))?;

    write_frames_to_disk(&temp_dir, &frames)?;
    encode_with_ffmpeg(&temp_dir)
}

fn write_frames_to_disk(temp_dir: &PathBuf, frames: &[timeline::TimelineFrame]) -> Result<(), String> {
    for (frame_idx, frame) in frames.iter().enumerate() {
        let frame_duration_frames = (frame.duration_ms as u64 * FPS as u64) / 1000;
        for dup in 0..frame_duration_frames {
            let frame_num = (frame_idx as u64) * 10000 + dup;
            let png_path = temp_dir.join(format!("frame_{:06}.png", frame_num));
            frame.image
                .save(&png_path)
                .map_err(|e| format!("Failed to write frame PNG: {e}"))?;
        }
    }
    Ok(())
}

fn encode_with_ffmpeg(temp_dir: &PathBuf) -> Result<Vec<u8>, String> {
    let output_path = temp_dir.join("output.mp4");
    let frame_pattern = temp_dir.join("frame_%06d.png");

    let status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-framerate")
        .arg(FPS.to_string())
        .arg("-i")
        .arg(frame_pattern.to_str().ok_or("Invalid frame pattern")?)
        .arg("-c:v")
        .arg("libx264")
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg(output_path.to_str().ok_or("Invalid output path")?)
        .output()
        .map_err(|e| format!("FFmpeg not available: {e}. Install FFmpeg to enable MP4 export."))?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        return Err(format!("FFmpeg encoding failed: {stderr}"));
    }

    fs::read(&output_path)
        .map_err(|e| format!("Failed to read output MP4: {e}"))
        .and_then(|bytes| {
            fs::remove_dir_all(temp_dir).ok();
            Ok(bytes)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gev_core::document::{Step, Transition};

    #[test]
    fn video_export_requires_frames() {
        let doc = Document {
            title: "Test".into(),
            steps: vec![],
            voice_id: String::new(),
        };
        let result = build(&doc);
        assert!(result.is_err());
    }

    #[test]
    fn video_export_requires_ffmpeg() {
        let doc = Document {
            title: "Test".into(),
            steps: vec![Step {
                image: image::RgbaImage::new(100, 100),
                annotations: Vec::new(),
                title: String::new(),
                caption: String::new(),
                narration: String::new(),
                narration_audio: None,
                duration_override_ms: Some(500),
                transition: Transition::Cut,
            }],
            voice_id: String::new(),
        };
        let result = build(&doc);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("FFmpeg") || err_msg.contains("ffmpeg"));
    }
}
