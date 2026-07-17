//! Panneau latéral de légende : un texte par pastille, copie Markdown,
//! génération par IA.

use eframe::egui;

use super::{GevshotApp, Stage};
use crate::ui::widgets;

impl GevshotApp {
    pub(super) fn legend_panel_ui(&mut self, root: &mut egui::Ui) {
        if !self.legend_open || !matches!(self.stage, Stage::Editing { .. }) {
            return;
        }
        egui::Panel::right(egui::Id::new("legend"))
            .default_size(300.0)
            .show(root, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("Légende").size(16.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if widgets::icon_button(ui, false, widgets::icon_close)
                            .on_hover_text("Fermer")
                            .clicked()
                        {
                            self.legend_open = false;
                        }
                    });
                });
                ui.separator();
                self.editor.legend_ui(ui);
                if self.editor.has_badges() {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if widgets::pill_button(ui, "Copier en Markdown", false)
                            .on_hover_text(
                                "1. …\u{a0} 2. …\u{a0} — à coller sous l'image dans la doc",
                            )
                            .clicked()
                        {
                            match crate::infra::export::copy_text(&self.editor.legend_markdown()) {
                                Ok(()) => self.set_status("Légende copiée en Markdown"),
                                Err(e) => self.set_error(e),
                            }
                        }
                        let can_ai = !self.settings.api_key.is_empty() && self.ai_rx.is_none();
                        let generate = ui
                            .add_enabled(can_ai, egui::Button::new("Générer par IA"))
                            .on_hover_text(
                                "Rédige le texte de chaque étape à partir de l'image annotée",
                            );
                        if self.settings.api_key.is_empty() {
                            generate.on_disabled_hover_text(
                                "Configurez la clé API dans Paramètres → IA",
                            );
                        } else if generate.clicked() {
                            self.start_ai_legend();
                        }
                    });
                }
            });
    }
}
