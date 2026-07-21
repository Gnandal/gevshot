//! Flux IA : lancement des appels (légende, annotation sur instruction)
//! et application des résultats reçus du thread d'arrière-plan.

use super::{GevshotApp, Stage};
use crate::infra::ai;

impl GevshotApp {
    /// Dépile le résultat d'un appel IA terminé et l'applique.
    pub(super) fn poll_ai(&mut self) {
        let Some(rx) = &self.ai_rx else { return };
        match rx.try_recv() {
            Ok(result) => {
                self.ai_rx = None;
                self.ai_busy = None;
                match result {
                    ai::AiResult::Legend(steps) => {
                        let n = steps.len();
                        self.editor.set_badge_labels(&steps);
                        self.legend_open = true;
                        self.set_status(format!("Légende générée par IA ({n} étapes)"));
                    }
                    ai::AiResult::Annotate(annotations) => {
                        let n = annotations.len();
                        self.editor.add_annotations(annotations);
                        self.set_status(format!("{n} annotation(s) générée(s) par IA"));
                    }
                    ai::AiResult::Error(e) => self.set_error(format!("IA : {e}")),
                }
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.ai_rx = None;
                self.ai_busy = None;
                self.set_error("IA : le traitement a échoué");
            }
        }
    }

    /// Génère les textes d'étape de la légende à partir de l'image annotée.
    pub(super) fn start_ai_legend(&mut self) {
        let Stage::Editing { capture } = &self.stage else {
            return;
        };
        // L'image annotée : le modèle voit les pastilles et leurs numéros.
        let image = crate::infra::export::flatten(&capture.image, self.editor.annotations());
        let numbers = self.editor.badge_numbers();
        let (tx, rx) = std::sync::mpsc::channel();
        ai::spawn_legend(
            self.settings.api_key.clone(),
            self.settings.ai_model.clone(),
            image,
            numbers,
            tx,
        );
        self.ai_rx = Some(rx);
        self.ai_busy = Some("Génération de la légende par IA…".to_string());
        self.error = None;
        self.status = None;
    }

    /// Génère des annotations à partir de l'instruction saisie.
    pub(super) fn start_ai_annotate(&mut self) {
        let Stage::Editing { capture } = &self.stage else {
            return;
        };
        let (tx, rx) = std::sync::mpsc::channel();
        ai::spawn_annotate(
            self.settings.api_key.clone(),
            self.settings.ai_model.clone(),
            capture.image.clone(),
            self.ai_prompt.trim().to_string(),
            ai::style_from_editor(&self.editor),
            tx,
        );
        self.ai_rx = Some(rx);
        self.ai_busy = Some("Annotation par IA en cours…".to_string());
        self.error = None;
        self.status = None;
    }
}
