//! Barre d'état : statut global, nombre d'étapes/durée totale, mode courant, format.

use eframe::egui;
use gev_ui::theme;

use super::{Mode, StudioApp};
use crate::domain::duration;

impl StudioApp {
    pub(super) fn status_bar_ui(&mut self, root: &mut egui::Ui) {
        egui::Panel::bottom(egui::Id::new("status_bar")).show(root, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(8.0);

                // Statut.
                let status = "Prêt";
                ui.label(
                    egui::RichText::new(status)
                        .size(11.0)
                        .color(theme::TEXT_WEAK),
                );

                ui.separator();

                // Comptage et durée.
                let count = self.doc.steps.len();
                let total_ms = if count > 0 {
                    duration::total_ms(&self.doc)
                } else {
                    0
                };
                let total_s = duration::fmt_secs(total_ms);
                let status_text = if count == 1 {
                    format!("1 étape · {total_s}")
                } else {
                    format!("{count} étapes · {total_s}")
                };
                ui.label(
                    egui::RichText::new(status_text)
                        .size(11.0)
                        .color(theme::TEXT_WEAK),
                );

                ui.separator();

                // Mode et format.
                let mode_text = self.mode.label();
                let format_text = match self.mode {
                    Mode::Document => "A4 · 210 × 297 mm",
                    Mode::Slides => "16:9 · 1920 × 1080 px",
                    Mode::Video => "16:9 · 1920 × 1080 px",
                };
                let mode_format = format!("{mode_text} · {format_text}");
                ui.label(
                    egui::RichText::new(mode_format)
                        .size(11.0)
                        .color(theme::TEXT_WEAK),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("GevShot Studio v1.0.0")
                            .size(9.0)
                            .color(theme::TEXT_WEAK),
                    );
                });
            });
            ui.add_space(4.0);
        });
    }
}
