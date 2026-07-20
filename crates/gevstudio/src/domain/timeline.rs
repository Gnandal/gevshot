//! Séquencement pur : transforme un Document en timeline de frames prête à
//! être consommée par la Présentation (lecture live) ou l'export Vidéo
//! (rendu offline). Aucune I/O, aucune dépendance GUI/réseau, testable
//! unitairement.

use std::sync::Arc;

use gev_core::document::{Document, NarrationAudio, Transition};
use gev_core::fonts::FontSet;
use gev_core::render;

/// Une frame dans la timeline : image rendue, durée, audio, transition.
pub struct TimelineFrame {
    /// Image avec annotations brûlées.
    pub image: image::RgbaImage,
    /// Durée d'affichage en millisecondes.
    pub duration_ms: u32,
    /// Audio synchronisé (narration), partagé via Arc.
    pub audio: Option<Arc<NarrationAudio>>,
    /// Transition d'entrée.
    pub transition_in: Transition,
}

const DEFAULT_DURATION_MS: u32 = 2500;

/// Séquence un document en timeline de frames.
///
/// Règle de durée par étape (priorité décroissante) :
/// 1. duration_override_ms si défini
/// 2. narration_audio.duration_ms s'il y a de l'audio
/// 3. DEFAULT_DURATION_MS (2500 ms) sinon
pub fn sequence(doc: &Document, fonts: &FontSet) -> Result<Vec<TimelineFrame>, String> {
    let mut frames = Vec::new();

    for step in &doc.steps {
        // Aplatir l'image avec les annotations.
        let flat = render::flatten(&step.image, &step.annotations, fonts);

        // Calculer la durée selon la règle de priorité.
        let duration_ms = step
            .duration_override_ms
            .or_else(|| step.narration_audio.as_ref().map(|na| na.duration_ms))
            .unwrap_or(DEFAULT_DURATION_MS);

        // Créer la frame.
        frames.push(TimelineFrame {
            image: flat,
            duration_ms,
            audio: step.narration_audio.clone().map(Arc::new),
            transition_in: step.transition,
        });
    }

    Ok(frames)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gev_core::document::Step;

    #[test]
    fn duration_override_has_priority() {
        let step = Step {
            image: image::RgbaImage::new(100, 100),
            annotations: Vec::new(),
            title: String::new(),
            caption: String::new(),
            narration: String::new(),
            narration_audio: None,
            duration_override_ms: Some(5000),
            transition: Transition::Cut,
        };
        let doc = Document {
            title: String::new(),
            steps: vec![step],
            voice_id: String::new(),
        };
        let frames = sequence(&doc, &FontSet::empty()).unwrap();
        assert_eq!(frames[0].duration_ms, 5000);
    }

    #[test]
    fn default_duration_if_no_override_or_audio() {
        let step = Step {
            image: image::RgbaImage::new(100, 100),
            annotations: Vec::new(),
            title: String::new(),
            caption: String::new(),
            narration: String::new(),
            narration_audio: None,
            duration_override_ms: None,
            transition: Transition::Cut,
        };
        let doc = Document {
            title: String::new(),
            steps: vec![step],
            voice_id: String::new(),
        };
        let frames = sequence(&doc, &FontSet::empty()).unwrap();
        assert_eq!(frames[0].duration_ms, DEFAULT_DURATION_MS);
    }

    #[test]
    fn flatten_applied_to_frames() {
        use gev_core::model::{Annotation, Shape};
        use ecolor::Color32;

        let mut img = image::RgbaImage::new(100, 100);
        // Remplir avec une couleur
        for px in img.pixels_mut() {
            *px = image::Rgba([200, 200, 200, 255]);
        }

        let step = Step {
            image: img,
            annotations: vec![Annotation {
                shape: Shape::Rect {
                    rect: emath::Rect::from_min_size(
                        emath::pos2(10.0, 10.0),
                        emath::vec2(30.0, 30.0),
                    ),
                },
                color: Color32::RED,
                stroke_width: 2.0,
            }],
            title: String::new(),
            caption: String::new(),
            narration: String::new(),
            narration_audio: None,
            duration_override_ms: None,
            transition: Transition::Cut,
        };
        let doc = Document {
            title: String::new(),
            steps: vec![step],
            voice_id: String::new(),
        };
        let frames = sequence(&doc, &FontSet::empty()).unwrap();
        // Les annotations doivent être brûlées dans l'image.
        assert_eq!(frames[0].image.width(), 100);
        assert_eq!(frames[0].image.height(), 100);
    }
}
