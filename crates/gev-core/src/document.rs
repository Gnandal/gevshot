//! Document GEVSTUDIO : une séquence d'étapes (capture annotée + titre +
//! légende) assemblées en un guide pas-à-pas, sérialisée au format `.gevd`.
//! Le rendu de chaque étape réutilise `render::flatten` ; l'export (PDF,
//! PPTX…) vit côté application.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::model::Annotation;
use crate::project::{self, AnnotationData};

pub const EXTENSION: &str = "gevd";
const VERSION: u32 = 1;

#[derive(Clone)]
pub struct NarrationAudio {
    pub bytes: Vec<u8>,
    pub format: AudioFormat,
    pub duration_ms: u32,
    pub text_hash: u64,
    pub voice_id: String,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum AudioFormat {
    Mp3,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Transition {
    Cut,
    Fade,
}

impl Default for Transition {
    fn default() -> Self {
        Transition::Cut
    }
}

/// Une étape du guide : la capture, ses annotations, et le texte associé.
#[derive(Clone)]
pub struct Step {
    pub image: image::RgbaImage,
    pub annotations: Vec<Annotation>,
    /// Titre court de l'étape (peut être vide).
    pub title: String,
    /// Texte descriptif sous l'étape (peut être vide).
    pub caption: String,
    /// Texte de narration (voix off). Vide = pas de narration pour cette étape.
    pub narration: String,
    /// Audio synthétisé (cache), régénéré si `narration` change.
    pub narration_audio: Option<NarrationAudio>,
    /// Durée explicite si l'utilisateur veut surcharger le calcul auto.
    pub duration_override_ms: Option<u32>,
    /// Transition d'entrée vers cette étape.
    pub transition: Transition,
}

/// Un document complet : un titre et une suite ordonnée d'étapes.
#[derive(Clone)]
pub struct Document {
    pub title: String,
    pub steps: Vec<Step>,
    /// Voix par défaut pour les nouvelles étapes.
    pub voice_id: String,
}

// Format sur disque : mêmes types découplés d'egui que `project`, réutilisant
// son codec image et sa sérialisation d'annotations.

#[derive(Serialize, Deserialize)]
struct NarrationAudioData {
    #[serde(default)]
    bytes_base64: String,
    format: AudioFormat,
    duration_ms: u32,
    text_hash: u64,
    #[serde(default)]
    voice_id: String,
}

#[derive(Serialize, Deserialize)]
struct StepData {
    #[serde(default)]
    title: String,
    #[serde(default)]
    caption: String,
    image_png_base64: String,
    annotations: Vec<AnnotationData>,
    #[serde(default)]
    narration: String,
    #[serde(default)]
    narration_audio: Option<NarrationAudioData>,
    #[serde(default)]
    duration_override_ms: Option<u32>,
    #[serde(default)]
    transition: Transition,
}

#[derive(Serialize, Deserialize)]
struct DocumentFile {
    version: u32,
    #[serde(default)]
    title: String,
    steps: Vec<StepData>,
    #[serde(default)]
    voice_id: String,
}

pub fn save(path: &Path, doc: &Document) -> Result<(), String> {
    let steps = doc
        .steps
        .iter()
        .map(|s| {
            Ok(StepData {
                title: s.title.clone(),
                caption: s.caption.clone(),
                image_png_base64: project::encode_png(&s.image)?,
                annotations: s.annotations.iter().map(project::to_data).collect(),
                narration: s.narration.clone(),
                narration_audio: s.narration_audio.as_ref().map(|na| {
                    use base64::Engine;
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&na.bytes);
                    NarrationAudioData {
                        bytes_base64: b64,
                        format: na.format,
                        duration_ms: na.duration_ms,
                        text_hash: na.text_hash,
                        voice_id: na.voice_id.clone(),
                    }
                }),
                duration_override_ms: s.duration_override_ms,
                transition: s.transition,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let file = DocumentFile {
        version: VERSION,
        title: doc.title.clone(),
        steps,
        voice_id: doc.voice_id.clone(),
    };
    let json =
        serde_json::to_string(&file).map_err(|e| format!("Sérialisation impossible : {e}"))?;
    std::fs::write(path, json).map_err(|e| format!("Écriture impossible : {e}"))
}

pub fn load(path: &Path) -> Result<Document, String> {
    let json = std::fs::read_to_string(path).map_err(|e| format!("Lecture impossible : {e}"))?;
    let file: DocumentFile =
        serde_json::from_str(&json).map_err(|e| format!("Document invalide : {e}"))?;
    if file.version > VERSION {
        return Err(format!(
            "Document créé avec une version plus récente (v{})",
            file.version
        ));
    }
    let steps = file
        .steps
        .iter()
        .map(|s| {
            use base64::Engine;
            Ok(Step {
                image: project::decode_png(&s.image_png_base64)?,
                annotations: s.annotations.iter().map(project::from_data).collect(),
                title: s.title.clone(),
                caption: s.caption.clone(),
                narration: s.narration.clone(),
                narration_audio: s.narration_audio.as_ref().map(|nad| {
                    let bytes = base64::engine::general_purpose::STANDARD
                        .decode(&nad.bytes_base64)
                        .unwrap_or_default();
                    NarrationAudio {
                        bytes,
                        format: nad.format,
                        duration_ms: nad.duration_ms,
                        text_hash: nad.text_hash,
                        voice_id: nad.voice_id.clone(),
                    }
                }),
                duration_override_ms: s.duration_override_ms,
                transition: s.transition,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(Document {
        title: file.title,
        steps,
        voice_id: file.voice_id,
    })
}
