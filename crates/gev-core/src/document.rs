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

/// Une étape du guide : la capture, ses annotations, et le texte associé.
pub struct Step {
    pub image: image::RgbaImage,
    pub annotations: Vec<Annotation>,
    /// Titre court de l'étape (peut être vide).
    pub title: String,
    /// Texte descriptif sous l'étape (peut être vide).
    pub caption: String,
}

/// Un document complet : un titre et une suite ordonnée d'étapes.
pub struct Document {
    pub title: String,
    pub steps: Vec<Step>,
}

// Format sur disque : mêmes types découplés d'egui que `project`, réutilisant
// son codec image et sa sérialisation d'annotations.

#[derive(Serialize, Deserialize)]
struct StepData {
    #[serde(default)]
    title: String,
    #[serde(default)]
    caption: String,
    image_png_base64: String,
    annotations: Vec<AnnotationData>,
}

#[derive(Serialize, Deserialize)]
struct DocumentFile {
    version: u32,
    #[serde(default)]
    title: String,
    steps: Vec<StepData>,
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
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let file = DocumentFile {
        version: VERSION,
        title: doc.title.clone(),
        steps,
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
            Ok(Step {
                image: project::decode_png(&s.image_png_base64)?,
                annotations: s.annotations.iter().map(project::from_data).collect(),
                title: s.title.clone(),
                caption: s.caption.clone(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(Document {
        title: file.title,
        steps,
    })
}
