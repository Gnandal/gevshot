//! Panneau gauche « Étapes · N » : cartes d'étapes réordonnables, actions
//! sur l'étape sélectionnée, bouton d'import.

use eframe::egui;
use gev_ui::{theme, widgets};

use super::StudioApp;
use crate::domain::duration;
use crate::ui::icons;

/// Action demandée sur une étape (appliquée après la boucle d'affichage).
enum Action {
    Select(usize),
    Up,
    Down,
    Duplicate,
    Delete,
}

impl StudioApp {
    pub(super) fn steps_panel_ui(&mut self, root: &mut egui::Ui) {
        egui::Panel::left(egui::Id::new("steps_panel"))
            .default_size(266.0)
            .show(root, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(format!("ÉTAPES · {}", self.doc.steps.len()))
                            .size(11.0)
                            .strong()
                            .color(theme::SUB),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(4.0);
                        if widgets::icon_button(ui, false, icons::icon_plus)
                            .on_hover_text("Importer des captures")
                            .clicked()
                        {
                            self.import_captures();
                        }
                    });
                });
                ui.separator();

                let mut action: Option<Action> = None;
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(4.0);
                        for i in 0..self.doc.steps.len() {
                            if let Some(a) = self.step_card_ui(ui, i) {
                                action = Some(a);
                            }
                            ui.add_space(4.0);
                        }
                        // Bouton d'import en pointillés.
                        ui.add_space(2.0);
                        if self.import_button_ui(ui) {
                            action = None;
                            self.import_captures();
                        }
                        ui.add_space(12.0);
                    });

                match action {
                    Some(Action::Select(i)) => self.select(i),
                    Some(Action::Up) => self.move_selected(-1),
                    Some(Action::Down) => self.move_selected(1),
                    Some(Action::Duplicate) => self.duplicate_selected(),
                    Some(Action::Delete) => self.delete_selected(),
                    None => {}
                }
            });
    }

    /// Carte d'une étape ; renvoie l'action cliquée s'il y en a une.
    fn step_card_ui(&mut self, ui: &mut egui::Ui, index: usize) -> Option<Action> {
        let step = &self.doc.steps[index];
        let active = index == self.selected;
        let title = if step.title.trim().is_empty() {
            "(sans titre)".to_string()
        } else {
            step.title.clone()
        };
        let meta = duration::fmt_secs(duration::step_duration_ms(step));
        let has_narration = !step.narration.trim().is_empty();
        let mut action = None;

        let frame = egui::Frame::default()
            .fill(if active {
                theme::ACCENT_SOFT
            } else {
                egui::Color32::TRANSPARENT
            })
            .stroke(egui::Stroke::new(
                1.0,
                if active {
                    theme::ACCENT
                } else {
                    egui::Color32::TRANSPARENT
                },
            ))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(9, 8));

        let inner = frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                let (badge_rect, _) =
                    ui.allocate_exact_size(egui::vec2(22.0, 22.0), egui::Sense::hover());
                icons::step_badge(ui.painter(), badge_rect, index + 1, active);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.label(
                        egui::RichText::new(title)
                            .size(12.5)
                            .color(theme::TEXT),
                    );
                    ui.horizontal(|ui| {
                        let (rect, _) = ui
                            .allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                        icons::icon_clock(ui.painter(), rect.center(), theme::SUB);
                        ui.label(egui::RichText::new(meta).size(10.5).color(theme::SUB));
                    });
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(15.0, 15.0), egui::Sense::hover());
                    let color = if has_narration {
                        theme::ACCENT
                    } else {
                        theme::BORDER
                    };
                    icons::icon_waveform(ui.painter(), rect.center(), color);
                });
            });

            // Rangée d'actions, sur la carte sélectionnée seulement.
            if active {
                ui.add_space(2.0);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 2.0;
                    if widgets::icon_button(ui, false, icons::icon_trash)
                        .on_hover_text("Supprimer")
                        .clicked()
                    {
                        action = Some(Action::Delete);
                    }
                    if widgets::icon_button(ui, false, icons::icon_duplicate)
                        .on_hover_text("Dupliquer")
                        .clicked()
                    {
                        action = Some(Action::Duplicate);
                    }
                    if widgets::icon_button_enabled(
                        ui,
                        index + 1 < self.doc.steps.len(),
                        icons::icon_down,
                    )
                    .on_hover_text("Descendre")
                    .clicked()
                    {
                        action = Some(Action::Down);
                    }
                    if widgets::icon_button_enabled(ui, index > 0, icons::icon_up)
                        .on_hover_text("Monter")
                        .clicked()
                    {
                        action = Some(Action::Up);
                    }
                });
            }
        });

        // Clic sur la carte (hors boutons d'action) : sélection.
        let response = inner
            .response
            .interact(egui::Sense::click());
        if action.is_none() && response.clicked() {
            action = Some(Action::Select(index));
        }
        action
    }

    /// Bouton « Importer une capture » en pointillés ; renvoie true si cliqué.
    fn import_button_ui(&mut self, ui: &mut egui::Ui) -> bool {
        let (rect, resp) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 40.0),
            egui::Sense::click(),
        );
        let hovered = resp.hovered();
        let color = if hovered { theme::ACCENT } else { theme::TEXT_WEAK };
        // Bordure en tirets.
        let stroke = egui::Stroke::new(1.5, if hovered { theme::ACCENT } else { theme::BORDER });
        let r = rect.shrink(1.0);
        for segment in dashed_rect(r, 6.0, 4.0) {
            ui.painter().line_segment(segment, stroke);
        }
        let (icon_rect, _) = (egui::Rect::from_center_size(
            rect.center() - egui::vec2(74.0, 0.0),
            egui::vec2(16.0, 16.0),
        ), ());
        icons::icon_image_plus(ui.painter(), icon_rect.center(), color);
        ui.painter().text(
            rect.center() + egui::vec2(10.0, 0.0),
            egui::Align2::CENTER_CENTER,
            "Importer une capture",
            egui::FontId::proportional(12.0),
            color,
        );
        resp.clicked()
    }
}

/// Segments d'un rectangle en tirets (coins arrondis ignorés).
fn dashed_rect(rect: egui::Rect, dash: f32, gap: f32) -> Vec<[egui::Pos2; 2]> {
    let mut out = Vec::new();
    let mut edge = |from: egui::Pos2, to: egui::Pos2| {
        let len = from.distance(to);
        let dir = (to - from) / len;
        let mut t = 0.0;
        while t < len {
            let end = (t + dash).min(len);
            out.push([from + dir * t, from + dir * end]);
            t += dash + gap;
        }
    };
    edge(rect.left_top(), rect.right_top());
    edge(rect.right_top(), rect.right_bottom());
    edge(rect.right_bottom(), rect.left_bottom());
    edge(rect.left_bottom(), rect.left_top());
    out
}
