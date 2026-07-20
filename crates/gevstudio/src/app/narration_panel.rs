//! Panneau d'édition narration pour l'étape sélectionnée.
//! Visible en mode Document/Présentation/Vidéo.
//!
//! Fonctionnalités :
//! - TextEdit du texte narration
//! - Bouton « Utiliser la légende » (copie caption → narration)
//! - Bouton « Générer la voix » (stub, locked si pas Pro)

use gev_core::document::Step;

/// État du panneau narration.
pub struct NarrationPanelState {
    /// Texte en édition (peut différer du Step.narration si non validé).
    pub edit_text: String,
    /// Index de l'étape en édition.
    pub editing_step: Option<usize>,
    /// Message d'erreur/succès temporaire.
    pub message: Option<String>,
}

impl Default for NarrationPanelState {
    fn default() -> Self {
        Self {
            edit_text: String::new(),
            editing_step: None,
            message: None,
        }
    }
}

impl NarrationPanelState {
    /// Commence l'édition d'une étape.
    pub fn edit_step(&mut self, step: &Step, index: usize) {
        self.edit_text = step.narration.clone();
        self.editing_step = Some(index);
        self.message = None;
    }

    /// Valide et applique la narration à l'étape.
    pub fn apply(&mut self, step: &mut Step) {
        step.narration = self.edit_text.clone();
        self.message = Some("Narration mise à jour".to_string());
    }

    /// Copie la légende dans le texte de narration.
    pub fn use_caption(&mut self, caption: &str) {
        self.edit_text = caption.to_string();
        self.message = Some("Légende copiée dans la narration".to_string());
    }

    /// Lance la génération de voix (futur).
    pub fn generate_voice_requested(&self) -> bool {
        // v1 : stub
        // v2 : lancer infra::tts::spawn_synthesize()
        false
    }

    /// Réinitialise le message d'état.
    pub fn clear_message(&mut self) {
        self.message = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gev_core::document::Transition;

    #[test]
    fn narration_panel_edit_step() {
        let step = Step {
            image: image::RgbaImage::new(100, 100),
            annotations: Vec::new(),
            title: String::new(),
            caption: "Original caption".into(),
            narration: "Original narration".into(),
            narration_audio: None,
            duration_override_ms: None,
            transition: Transition::Cut,
        };

        let mut panel = NarrationPanelState::default();
        panel.edit_step(&step, 0);

        assert_eq!(panel.editing_step, Some(0));
        assert_eq!(panel.edit_text, "Original narration");
    }

    #[test]
    fn narration_panel_apply() {
        let mut step = Step {
            image: image::RgbaImage::new(100, 100),
            annotations: Vec::new(),
            title: String::new(),
            caption: String::new(),
            narration: String::new(),
            narration_audio: None,
            duration_override_ms: None,
            transition: Transition::Cut,
        };

        let mut panel = NarrationPanelState::default();
        panel.edit_step(&step, 0);
        panel.edit_text = "New narration".into();
        panel.apply(&mut step);

        assert_eq!(step.narration, "New narration");
        assert!(panel.message.is_some());
    }

    #[test]
    fn narration_panel_use_caption() {
        let mut panel = NarrationPanelState::default();
        panel.use_caption("Caption text");

        assert_eq!(panel.edit_text, "Caption text");
        assert!(panel.message.is_some());
    }
}
