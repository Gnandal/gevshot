//! Export vidéo MP4 : orchestration FFmpeg (v1 implémentation future).
//! Pipeline :
//! 1. Séquencer le document en frames
//! 2. Écrire chaque frame PNG dans tempdir (dupliquer selon durée pour 30 fps)
//! 3. Concaténer pistes audio (silence pour étapes sans narration)
//! 4. Invoquer ffmpeg pour encoder PNG + audio → MP4
//! 5. Retourner octets (pattern identique à pdf/pptx/gif)

use gev_core::document::Document;

/// Génère un MP4 depuis un document (v1 : stub).
pub fn build(_doc: &Document) -> Result<Vec<u8>, String> {
    Err("MP4 export not implemented in v1".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gev_core::document::{Step, Transition};

    #[test]
    fn video_export_not_implemented() {
        let doc = Document {
            title: "Test".into(),
            steps: vec![Step {
                image: image::RgbaImage::new(100, 100),
                annotations: Vec::new(),
                title: String::new(),
                caption: String::new(),
                narration: String::new(),
                narration_audio: None,
                duration_override_ms: None,
                transition: Transition::Cut,
            }],
            voice_id: String::new(),
        };
        let result = build(&doc);
        assert!(result.is_err());
    }
}
