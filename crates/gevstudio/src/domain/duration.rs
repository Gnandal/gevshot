//! Durée d'affichage des étapes : priorité à la surcharge manuelle, puis à
//! la durée de l'audio de narration, sinon la valeur par défaut.

use gev_core::document::{Document, Step};

/// Durée par défaut d'une étape sans narration ni surcharge.
pub const DEFAULT_DURATION_MS: u32 = 2500;

pub fn step_duration_ms(step: &Step) -> u32 {
    step.duration_override_ms
        .or(step.narration_audio.as_ref().map(|a| a.duration_ms))
        .unwrap_or(DEFAULT_DURATION_MS)
}

pub fn total_ms(doc: &Document) -> u32 {
    doc.steps.iter().map(step_duration_ms).sum()
}

/// Durée au format d'affichage français : `3,5 s`.
pub fn fmt_secs(ms: u32) -> String {
    format!("{:.1} s", ms as f32 / 1000.0).replace('.', ",")
}

/// Temps au format transport : `00:07`.
pub fn fmt_clock(ms: u32) -> String {
    let total = ms / 1000;
    format!("{:02}:{:02}", total / 60, total % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gev_core::document::{AudioFormat, NarrationAudio, Transition};

    fn step() -> Step {
        Step {
            image: image::RgbaImage::new(4, 4),
            annotations: Vec::new(),
            title: String::new(),
            caption: String::new(),
            narration: String::new(),
            narration_audio: None,
            duration_override_ms: None,
            transition: Transition::Cut,
        }
    }

    #[test]
    fn priority_override_then_audio_then_default() {
        let mut s = step();
        assert_eq!(step_duration_ms(&s), DEFAULT_DURATION_MS);
        s.narration_audio = Some(NarrationAudio {
            bytes: Vec::new(),
            format: AudioFormat::Mp3,
            duration_ms: 4200,
            text_hash: 0,
            voice_id: String::new(),
        });
        assert_eq!(step_duration_ms(&s), 4200);
        s.duration_override_ms = Some(1500);
        assert_eq!(step_duration_ms(&s), 1500);
    }

    #[test]
    fn formats() {
        assert_eq!(fmt_secs(3500), "3,5 s");
        assert_eq!(fmt_clock(67_000), "01:07");
    }
}
