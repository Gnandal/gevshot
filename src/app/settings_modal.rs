//! Modale des Paramètres, affichée dans la fenêtre principale.

use eframe::egui;

use super::GevshotApp;
use crate::ui::settings::{self, SettingsAction, SettingsDraft};
use crate::ui::widgets;

impl GevshotApp {
    pub(super) fn show_settings_window(&mut self, ctx: &egui::Context) {
        if self.settings_draft.is_none() {
            self.settings_draft = Some(SettingsDraft::new(&self.settings));
        }
        let mut action = SettingsAction::None;
        let mut close_requested = false;
        // Modale dans la fenêtre principale : fond assombri, Échap ou clic
        // à l'extérieur pour fermer.
        let modal = egui::Modal::new(egui::Id::new("settings_modal")).show(ctx, |ui| {
            let avail = ui.ctx().content_rect();
            ui.set_width((avail.width() - 120.0).clamp(520.0, 760.0));
            ui.set_height((avail.height() - 140.0).clamp(360.0, 520.0));
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("Paramètres").size(17.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if widgets::icon_button(ui, false, widgets::icon_close)
                        .on_hover_text("Fermer")
                        .clicked()
                    {
                        close_requested = true;
                    }
                });
            });
            ui.separator();
            if let Some(draft) = self.settings_draft.as_mut() {
                action = settings::show(ui, draft);
            }
        });
        if modal.should_close() {
            close_requested = true;
        }
        match action {
            SettingsAction::Apply => {
                if let Some(draft) = self.settings_draft.take() {
                    self.settings = draft.settings;
                    let [r, g, b, a] = self.settings.default_color;
                    self.editor.set_defaults(
                        egui::Color32::from_rgba_unmultiplied(r, g, b, a),
                        self.settings.default_stroke,
                    );
                    if let Err(e) = self.settings.save() {
                        self.set_error(e);
                    } else {
                        self.set_status("Paramètres enregistrés");
                    }
                }
                self.settings_open = false;
            }
            SettingsAction::Cancel => {
                self.settings_draft = None;
                self.settings_open = false;
            }
            SettingsAction::None => {
                if close_requested {
                    self.settings_draft = None;
                    self.settings_open = false;
                }
            }
        }
    }
}
