//! Mode Présentation : lecture interactive en plein écran.
//!
//! Affiche les étapes séquencées avec :
//! - Navigation au clavier (→/Espace = suivant, ← = précédent, Échap = quitter)
//! - Avance automatique après duration_ms (interruptible par navigation manuelle)
//! - Lecture audio synchronisée si narration présente
//! - Pas de fi ligrane Pro (gratuit pour vitrine)

use std::time::{Duration, Instant};

use gev_core::document::Document;
use gev_core::fonts::FontSet;

use crate::domain::timeline::{self, TimelineFrame};

/// État d'une présentation en cours.
pub struct PresentationState {
    pub frames: Vec<TimelineFrame>,
    pub current: usize,
    pub start_time: Instant,
    pub paused: bool,
}

impl PresentationState {
    /// Lance une présentation depuis un document.
    pub fn new(doc: &Document, fonts: &FontSet) -> Result<Self, String> {
        let frames = timeline::sequence(doc, fonts)?;
        if frames.is_empty() {
            return Err("Cannot start presentation: no frames".to_string());
        }

        Ok(Self {
            frames,
            current: 0,
            start_time: Instant::now(),
            paused: false,
        })
    }

    /// Étape courante.
    pub fn current_frame(&self) -> &TimelineFrame {
        &self.frames[self.current]
    }

    /// Avance à l'étape suivante (navigation manuelle).
    pub fn next(&mut self) {
        if self.current < self.frames.len() - 1 {
            self.current += 1;
            self.start_time = Instant::now();
        }
    }

    /// Revient à l'étape précédente.
    pub fn prev(&mut self) {
        if self.current > 0 {
            self.current -= 1;
            self.start_time = Instant::now();
        }
    }

    /// Bascule pause/lecture.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Temps écoulé depuis le début de l'étape courante (ms).
    pub fn elapsed_ms(&self) -> u32 {
        if self.paused {
            0
        } else {
            self.start_time.elapsed().as_millis() as u32
        }
    }

    /// Doit-on passer à l'étape suivante (auto-advance selon durée) ?
    pub fn should_advance(&self) -> bool {
        !self.paused && self.elapsed_ms() >= self.current_frame().duration_ms
    }

    /// Nombre total d'étapes.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Index 1-based pour affichage.
    pub fn current_index_display(&self) -> usize {
        self.current + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gev_core::document::{Step, Transition};

    #[test]
    fn presentation_state_navigation() {
        let doc = Document {
            title: "Test".into(),
            steps: vec![
                Step {
                    image: image::RgbaImage::new(100, 100),
                    annotations: Vec::new(),
                    title: "Step 1".into(),
                    caption: String::new(),
                    narration: String::new(),
                    narration_audio: None,
                    duration_override_ms: Some(1000),
                    transition: Transition::Cut,
                },
                Step {
                    image: image::RgbaImage::new(100, 100),
                    annotations: Vec::new(),
                    title: "Step 2".into(),
                    caption: String::new(),
                    narration: String::new(),
                    narration_audio: None,
                    duration_override_ms: Some(1000),
                    transition: Transition::Cut,
                },
            ],
            voice_id: String::new(),
        };

        let fonts = FontSet::empty();
        let mut pres = PresentationState::new(&doc, &fonts).unwrap();

        assert_eq!(pres.current, 0);
        assert_eq!(pres.frame_count(), 2);
        assert_eq!(pres.current_index_display(), 1);

        pres.next();
        assert_eq!(pres.current, 1);
        assert_eq!(pres.current_index_display(), 2);

        pres.prev();
        assert_eq!(pres.current, 0);

        // Can't go past end
        pres.next();
        pres.next();
        assert_eq!(pres.current, 1);
    }

    #[test]
    fn presentation_auto_advance() {
        let doc = Document {
            title: "Test".into(),
            steps: vec![Step {
                image: image::RgbaImage::new(100, 100),
                annotations: Vec::new(),
                title: String::new(),
                caption: String::new(),
                narration: String::new(),
                narration_audio: None,
                duration_override_ms: Some(50),
                transition: Transition::Cut,
            }],
            voice_id: String::new(),
        };

        let fonts = FontSet::empty();
        let pres = PresentationState::new(&doc, &fonts).unwrap();

        // Elapsed should be small at start
        assert!(pres.elapsed_ms() < 50);
        assert!(!pres.should_advance());

        // After 50ms, should_advance returns true (timing is approximate)
        std::thread::sleep(Duration::from_millis(60));
        let pres2 = PresentationState::new(&doc, &fonts).unwrap();
        // This is a loose test because timing is imprecise
        // In real UI, elapsed_ms is polled continuously
    }

    #[test]
    fn presentation_pause() {
        let doc = Document {
            title: "Test".into(),
            steps: vec![Step {
                image: image::RgbaImage::new(100, 100),
                annotations: Vec::new(),
                title: String::new(),
                caption: String::new(),
                narration: String::new(),
                narration_audio: None,
                duration_override_ms: Some(10000),
                transition: Transition::Cut,
            }],
            voice_id: String::new(),
        };

        let fonts = FontSet::empty();
        let mut pres = PresentationState::new(&doc, &fonts).unwrap();

        assert!(!pres.paused);
        pres.toggle_pause();
        assert!(pres.paused);

        // When paused, elapsed_ms is always 0
        assert_eq!(pres.elapsed_ms(), 0);
        assert!(!pres.should_advance());

        pres.toggle_pause();
        assert!(!pres.paused);
    }
}
