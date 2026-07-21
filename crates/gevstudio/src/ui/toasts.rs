//! Notification éphémère centrée en bas de fenêtre (~2,6 s), dans l'esprit
//! de la maquette : carte blanche, icône accent, texte court.

use std::time::{Duration, Instant};

use eframe::egui;
use gev_ui::theme;

const TTL: Duration = Duration::from_millis(2600);

#[derive(Default)]
pub struct Toasts {
    current: Option<(String, Instant)>,
}

impl Toasts {
    pub fn push(&mut self, msg: impl Into<String>) {
        self.current = Some((msg.into(), Instant::now()));
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        let Some((msg, since)) = &self.current else {
            return;
        };
        if since.elapsed() >= TTL {
            self.current = None;
            return;
        }
        let msg = msg.clone();
        egui::Area::new(egui::Id::new("gevstudio_toast"))
            .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -46.0))
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::default()
                    .fill(theme::BG_TOOLBAR)
                    .stroke(egui::Stroke::new(1.0, theme::BORDER))
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(16, 10))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(16.0, 16.0),
                                egui::Sense::hover(),
                            );
                            crate::ui::icons::icon_check_circle(
                                ui.painter(),
                                rect.center(),
                                theme::ACCENT,
                            );
                            ui.label(egui::RichText::new(msg).size(12.5).color(theme::TEXT));
                        });
                    });
            });
        // Redessine tant que le toast est visible (pour l'expiration).
        ctx.request_repaint_after(Duration::from_millis(200));
    }
}
