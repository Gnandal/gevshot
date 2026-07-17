//! Barre d'état : dimensions et zoom à gauche, message central
//! (erreur > IA en cours > statut > astuce contextuelle), compteur à droite.

use eframe::egui;

use super::{GevshotApp, Stage};
use crate::ui::theme;

impl GevshotApp {
    pub(super) fn status_bar_ui(&mut self, root: &mut egui::Ui) {
        egui::Panel::bottom(egui::Id::new("status")).show(root, |ui| {
            let (rect, _) = ui
                .allocate_exact_size(egui::vec2(ui.available_width(), 26.0), egui::Sense::hover());
            let p = ui.painter();
            let font = egui::FontId::proportional(13.0);

            // Textes latéraux, mesurés pour réserver l'espace du centre.
            let mut left_end = rect.min.x + 10.0;
            let mut right_start = rect.max.x - 10.0;
            if let Stage::Editing { capture } = &self.stage {
                let dims = format!(
                    "{} × {} px · {:.0} %",
                    capture.image.width(),
                    capture.image.height(),
                    self.editor.zoom() * 100.0
                );
                let r = p.text(
                    egui::pos2(rect.min.x + 10.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    dims,
                    font.clone(),
                    theme::TEXT_WEAK,
                );
                left_end = r.max.x;
                let count = self.editor.annotation_count();
                let r = p.text(
                    egui::pos2(rect.max.x - 10.0, rect.center().y),
                    egui::Align2::RIGHT_CENTER,
                    format!("{count} annotation{}", if count > 1 { "s" } else { "" }),
                    font.clone(),
                    theme::TEXT_WEAK,
                );
                right_start = r.min.x;
            }

            let (msg, color) = if let Some(e) = &self.error {
                (e.clone(), theme::DANGER)
            } else if let Some(busy) = &self.ai_busy {
                (busy.clone(), theme::ACCENT)
            } else if let Some(s) = &self.status {
                (s.clone(), theme::SUCCESS)
            } else if matches!(self.stage, Stage::Editing { .. }) {
                (self.editor.hint().to_string(), theme::TEXT_WEAK)
            } else {
                ("Ctrl+Shift+S pour capturer".to_string(), theme::TEXT_WEAK)
            };

            // Message central tronqué (…) s'il ne tient pas entre les côtés.
            let available = (right_start - left_end - 32.0).max(0.0);
            if available > 20.0 {
                let mut job = egui::text::LayoutJob::simple_singleline(msg, font, color);
                job.wrap = egui::text::TextWrapping::truncate_at_width(available);
                let galley = p.layout_job(job);
                let center_x = (left_end + right_start) / 2.0;
                let pos = egui::pos2(
                    center_x - galley.size().x / 2.0,
                    rect.center().y - galley.size().y / 2.0,
                );
                p.galley(pos, galley, color);
            }
        });
    }
}
