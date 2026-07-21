//! Barre de titre : logo, menus, nom du projet + état d'enregistrement,
//! badge PRO et bouton Exporter.

use eframe::egui;
use gev_ui::theme;

use super::StudioApp;
use crate::ui::icons;

impl StudioApp {
    pub(super) fn title_bar_ui(&mut self, root: &mut egui::Ui) {
        egui::Panel::top(egui::Id::new("title_bar")).show(root, |ui| {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.add_space(10.0);

                // Logo + nom.
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
                ui.painter().rect_filled(
                    rect,
                    egui::CornerRadius::same(6),
                    theme::ACCENT,
                );
                icons::icon_movie(ui.painter(), rect.center(), egui::Color32::WHITE);
                ui.label(
                    egui::RichText::new("GevShot")
                        .size(13.0)
                        .strong()
                        .color(theme::TEXT),
                );
                ui.add_space(-4.0);
                ui.label(
                    egui::RichText::new("Studio")
                        .size(13.0)
                        .color(theme::TEXT_WEAK),
                );
                ui.add_space(6.0);
                ui.separator();

                // Menus.
                self.file_menu_ui(ui);
                self.edit_menu_ui(ui);
                self.insert_menu_ui(ui);
                self.help_menu_ui(ui);

                // Actions à droite.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    if gev_ui::widgets::pill_button(ui, "Exporter", true).clicked() {
                        self.toasts.push("Export disponible au prochain jalon");
                    }
                    if self.is_pro() {
                        // Badge PRO (tout est débloqué en FREE_MODE).
                        let text = egui::RichText::new("PRO")
                            .size(11.0)
                            .strong()
                            .color(theme::WARN_TEXT_ON);
                        egui::Frame::default()
                            .fill(theme::WARN)
                            .corner_radius(egui::CornerRadius::same(6))
                            .inner_margin(egui::Margin::symmetric(9, 4))
                            .show(ui, |ui| {
                                ui.label(text);
                            });
                    }

                    // Centre : nom du projet + état d'enregistrement.
                    ui.with_layout(
                        egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                        |ui| {
                            ui.horizontal(|ui| {
                                let w = ui.available_width();
                                let title = format!("Guide — {}", self.doc.title);
                                let state = if self.dirty { "Modifié" } else { "Enregistré" };
                                let galley_w = 260.0_f32.min(w);
                                ui.add_space((w - galley_w).max(0.0) / 2.0);
                                ui.label(
                                    egui::RichText::new(title)
                                        .size(12.5)
                                        .color(theme::TEXT_WEAK),
                                );
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(15.0, 15.0),
                                    egui::Sense::hover(),
                                );
                                let color = if self.dirty {
                                    theme::TEXT_WEAK
                                } else {
                                    theme::SUCCESS
                                };
                                icons::icon_cloud_check(ui.painter(), rect.center(), color);
                                ui.label(
                                    egui::RichText::new(state).size(10.5).color(theme::SUB),
                                );
                            });
                        },
                    );
                });
            });
            ui.add_space(5.0);
        });
    }

    fn file_menu_ui(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Fichier", |ui| {
            ui.set_min_width(220.0);
            if ui.button("Nouveau guide").clicked() {
                ui.close();
                self.new_doc();
            }
            if ui.button("Ouvrir…").clicked() {
                ui.close();
                self.open();
            }
            ui.separator();
            if ui.button("Enregistrer (Ctrl+S)").clicked() {
                ui.close();
                self.save();
            }
            if ui.button("Enregistrer sous…").clicked() {
                ui.close();
                self.save_as();
            }
            ui.separator();
            if ui.button("Importer des captures…").clicked() {
                ui.close();
                self.import_captures();
            }
        });
    }

    fn edit_menu_ui(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Édition", |ui| {
            ui.set_min_width(200.0);
            let has_steps = !self.doc.steps.is_empty();
            if ui
                .add_enabled(has_steps, egui::Button::new("Dupliquer l'étape"))
                .clicked()
            {
                ui.close();
                self.duplicate_selected();
            }
            if ui
                .add_enabled(has_steps, egui::Button::new("Supprimer l'étape"))
                .clicked()
            {
                ui.close();
                self.delete_selected();
            }
        });
    }

    fn insert_menu_ui(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Insérer", |ui| {
            ui.set_min_width(220.0);
            if ui.button("Importer des captures…").clicked() {
                ui.close();
                self.import_captures();
            }
        });
    }

    fn help_menu_ui(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("Aide", |ui| {
            ui.set_min_width(200.0);
            if ui.button("À propos de GevShot Studio").clicked() {
                ui.close();
                self.toasts
                    .push(format!("GevShot Studio v{}", env!("CARGO_PKG_VERSION")));
            }
        });
    }
}
