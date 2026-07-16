//! Sélecteur de fenêtre à capturer : liste modale des fenêtres ouvertes.

use eframe::egui;

use super::{CaptureMode, GevshotApp};

impl GevshotApp {
    pub(super) fn open_window_picker(&mut self) {
        match crate::infra::capture::list_windows() {
            Ok(windows) => {
                self.window_picker = Some(windows);
                self.error = None;
                self.status = None;
            }
            Err(e) => self.set_error(e),
        }
    }

    /// Fenêtre modale : liste des fenêtres capturables.
    pub(super) fn window_picker_ui(&mut self, ctx: &egui::Context) {
        let Some(windows) = self.window_picker.clone() else {
            return;
        };
        let mut open = true;
        let mut picked: Option<u32> = None;
        egui::Window::new("Choisir une fenêtre à capturer")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([440.0, 380.0])
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for w in &windows {
                            let label = if w.app_name.is_empty() {
                                w.title.clone()
                            } else {
                                format!("{} — {}", w.app_name, w.title)
                            };
                            if ui
                                .add(
                                    egui::Button::new(label)
                                        .wrap_mode(egui::TextWrapMode::Truncate)
                                        .min_size(egui::vec2(ui.available_width(), 30.0)),
                                )
                                .clicked()
                            {
                                picked = Some(w.id);
                            }
                        }
                    });
                ui.add_space(6.0);
                ui.weak("La fenêtre choisie est capturée telle qu'elle apparaît à l'écran.");
            });
        if let Some(id) = picked {
            self.window_picker = None;
            self.start_capture_mode(ctx, CaptureMode::WindowById(id), 0);
        } else if !open {
            self.window_picker = None;
        }
    }
}
