//! Barre de modes et d'outils : bascule Document/Slides/Vidéo, outils
//! d'annotation, zoom de la scène.

use eframe::egui;
use gev_ui::editor::Tool;
use gev_ui::{theme, widgets};

use super::{Mode, StudioApp};
use crate::ui::icons;

/// Fonction de dessin d'une icône.
type Icon = fn(&egui::Painter, egui::Pos2, egui::Color32);

impl StudioApp {
    pub(super) fn mode_bar_ui(&mut self, root: &mut egui::Ui) {
        egui::Panel::top(egui::Id::new("mode_bar")).show(root, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                self.mode_switch_ui(ui);
                ui.separator();
                self.tools_ui(ui);

                // Zoom à droite.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    if widgets::icon_button(ui, false, icons::icon_plus)
                        .on_hover_text("Zoomer")
                        .clicked()
                    {
                        self.zoom_pct = (self.zoom_pct + 10).min(200);
                    }
                    ui.label(
                        egui::RichText::new(format!("{} %", self.zoom_pct))
                            .size(12.0)
                            .color(theme::TEXT_WEAK),
                    );
                    if widgets::icon_button(ui, false, icon_minus)
                        .on_hover_text("Dézoomer")
                        .clicked()
                    {
                        self.zoom_pct = self.zoom_pct.saturating_sub(10).max(50);
                    }
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Bascule segmentée entre les trois modes.
    fn mode_switch_ui(&mut self, ui: &mut egui::Ui) {
        let modes: [(Mode, Icon); 3] = [
            (Mode::Document, icons::icon_doc as Icon),
            (Mode::Slides, icons::icon_slides as Icon),
            (Mode::Video, icons::icon_movie as Icon),
        ];
        egui::Frame::default()
            .fill(theme::BG_APP)
            .stroke(egui::Stroke::new(1.0, theme::BORDER))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(3))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                ui.horizontal(|ui| {
                    for (mode, icon) in modes {
                        let active = self.mode == mode;
                        let (fill, fg) = if active {
                            (theme::ACCENT, egui::Color32::WHITE)
                        } else {
                            (egui::Color32::TRANSPARENT, theme::TEXT_WEAK)
                        };
                        let text = egui::RichText::new(mode.label())
                            .size(12.5)
                            .strong()
                            .color(fg);
                        let resp = ui.add(
                            egui::Button::new(text)
                                .fill(fill)
                                .stroke(egui::Stroke::NONE)
                                .corner_radius(egui::CornerRadius::same(6))
                                .min_size(egui::vec2(96.0, 30.0)),
                        );
                        // Icône devant le libellé.
                        let icon_pos = resp.rect.left_center() + egui::vec2(16.0, 0.0);
                        icon(ui.painter(), icon_pos, fg);
                        if resp.clicked() {
                            self.mode = mode;
                        }
                    }
                });
            });
    }

    /// Outils d'annotation (actifs sur le canevas au jalon M2).
    fn tools_ui(&mut self, ui: &mut egui::Ui) {
        let tools: [(Tool, Icon, &str); 5] = [
            (Tool::Select, widgets::icon_select as Icon, "Sélection"),
            (Tool::Badge, widgets::icon_badge as Icon, "Pastille numérotée"),
            (Tool::Arrow, widgets::icon_arrow as Icon, "Flèche"),
            (Tool::Rect, widgets::icon_rect as Icon, "Cadre"),
            (Tool::Text, widgets::icon_text as Icon, "Texte"),
        ];
        for (tool, icon, tip) in tools {
            if widgets::icon_button(ui, self.tool == tool, icon)
                .on_hover_text(tip)
                .clicked()
            {
                self.tool = tool;
            }
        }
    }
}

fn icon_minus(p: &egui::Painter, c: egui::Pos2, color: egui::Color32) {
    p.line_segment(
        [c + egui::vec2(-5.5, 0.0), c + egui::vec2(5.5, 0.0)],
        egui::Stroke::new(1.8, color),
    );
}
