//! Sélection de région sur l'écran gelé : voile, rectangle, barre de modes,
//! validation ou annulation.

use eframe::egui;

use super::{Capture, CaptureMode, GevshotApp, Stage};
use crate::ui::theme;

/// Action choisie dans la barre de modes pendant la sélection.
enum ModeAction {
    /// Quitter l'overlay et ouvrir le sélecteur de fenêtre.
    PickWindow,
    /// Relancer une capture de région après 3 secondes.
    RegionIn3s,
    /// Annuler la capture (équivalent Échap).
    Cancel,
}

impl GevshotApp {
    pub(super) fn region_select_ui(&mut self, ctx: &egui::Context, root: &mut egui::Ui) {
        let (escape, enter) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::Enter),
            )
        });
        if escape {
            self.cancel_selection(ctx);
            return;
        }
        if enter {
            self.commit_region(ctx, None);
            return;
        }

        let mut committed: Option<Option<egui::Rect>> = None;
        let mut mode_action: Option<ModeAction> = None;
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(root, |ui| {
                let Stage::Selecting { full, .. } = &self.stage else {
                    return;
                };
                let rect = ui.max_rect();
                let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
                let painter = ui.painter();
                painter.image(
                    full.texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                if let Some(p) = response.interact_pointer_pos() {
                    if response.drag_started() {
                        self.selection = Some((p, p));
                    } else if response.dragged()
                        && let Some((_, cur)) = &mut self.selection
                    {
                        *cur = p;
                    }
                }

                let sel_rect = self.selection.map(|(a, b)| egui::Rect::from_two_pos(a, b));

                // Voile sombre autour de la sélection (ou partout si aucune).
                let veil = egui::Color32::from_black_alpha(120);
                match sel_rect {
                    Some(sel) => {
                        let sel = sel.intersect(rect);
                        for part in [
                            egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, sel.min.y)),
                            egui::Rect::from_min_max(egui::pos2(rect.min.x, sel.max.y), rect.max),
                            egui::Rect::from_min_max(
                                egui::pos2(rect.min.x, sel.min.y),
                                egui::pos2(sel.min.x, sel.max.y),
                            ),
                            egui::Rect::from_min_max(
                                egui::pos2(sel.max.x, sel.min.y),
                                egui::pos2(rect.max.x, sel.max.y),
                            ),
                        ] {
                            painter.rect_filled(part, 0.0, veil);
                        }
                        painter.rect_stroke(
                            sel,
                            egui::CornerRadius::ZERO,
                            egui::Stroke::new(1.5, egui::Color32::WHITE),
                            egui::StrokeKind::Outside,
                        );
                        // Dimensions en pixels image.
                        let scale = full.size().x / rect.width();
                        let dims = format!(
                            "{} × {}",
                            (sel.width() * scale).round() as u32,
                            (sel.height() * scale).round() as u32
                        );
                        painter.text(
                            sel.min + egui::vec2(0.0, -8.0),
                            egui::Align2::LEFT_BOTTOM,
                            dims,
                            egui::FontId::proportional(14.0),
                            egui::Color32::WHITE,
                        );
                    }
                    None => {
                        painter.rect_filled(rect, 0.0, veil);
                    }
                }

                mode_bar(ui.ctx(), &mut committed, &mut mode_action);

                if response.drag_stopped()
                    && let Some((a, b)) = self.selection.take()
                {
                    let sel = egui::Rect::from_two_pos(a, b).intersect(rect);
                    if sel.width() >= 5.0 && sel.height() >= 5.0 {
                        // Conversion écran → pixels image.
                        let scale = full.size().x / rect.width();
                        let img_rect = egui::Rect::from_min_max(
                            ((sel.min - rect.min) * scale).to_pos2(),
                            ((sel.max - rect.min) * scale).to_pos2(),
                        );
                        committed = Some(Some(img_rect));
                    }
                }

                response.on_hover_cursor(egui::CursorIcon::Crosshair);
            });

        if let Some(action) = mode_action {
            // `cancel_selection` remet `keep_annotations` à false : on le
            // préserve pour que « Recapturer le fond » survive au changement
            // de mode.
            let keep = self.keep_annotations;
            self.cancel_selection(ctx);
            match action {
                ModeAction::PickWindow => {
                    self.keep_annotations = keep;
                    self.open_window_picker();
                }
                ModeAction::RegionIn3s => {
                    self.keep_annotations = keep;
                    self.start_capture_mode(ctx, CaptureMode::Region, 3000);
                }
                ModeAction::Cancel => {}
            }
            return;
        }
        if let Some(region) = committed {
            self.commit_region(ctx, region);
        }
    }

    fn cancel_selection(&mut self, ctx: &egui::Context) {
        if let Stage::Selecting { prev, .. } = std::mem::replace(&mut self.stage, Stage::Empty)
            && let Some(capture) = prev
        {
            // L'éditeur n'a pas été réinitialisé : la session reprend telle quelle.
            self.stage = Stage::Editing { capture };
        }
        self.keep_annotations = false;
        self.selection = None;
        restore_window(ctx);
    }

    /// Termine la sélection : `region` en pixels image, `None` = écran entier.
    fn commit_region(&mut self, ctx: &egui::Context, region: Option<egui::Rect>) {
        let Stage::Selecting { full, .. } = std::mem::replace(&mut self.stage, Stage::Empty) else {
            return;
        };
        let image = match region {
            Some(r) => {
                let (iw, ih) = (full.image.width(), full.image.height());
                let x = (r.min.x.max(0.0) as u32).min(iw.saturating_sub(1));
                let y = (r.min.y.max(0.0) as u32).min(ih.saturating_sub(1));
                let w = ((r.width() as u32).max(1)).min(iw - x);
                let h = ((r.height() as u32).max(1)).min(ih - y);
                image::imageops::crop_imm(&full.image, x, y, w, h).to_image()
            }
            None => full.image,
        };
        let capture = Capture::from_image(ctx, image);
        self.stage = Stage::Editing { capture };
        if !std::mem::take(&mut self.keep_annotations) {
            self.editor.reset();
        }
        self.selection = None;
        self.doc_name = crate::infra::export::next_auto_name(&self.save_dir());
        self.update_title(ctx);
        self.copy_to_clipboard();
        restore_window(ctx);
    }
}

/// Barre de modes sombre, centrée en haut de l'écran pendant la sélection :
/// propose les différents modes de capture sans quitter l'overlay.
fn mode_bar(
    ctx: &egui::Context,
    committed: &mut Option<Option<egui::Rect>>,
    action: &mut Option<ModeAction>,
) {
    egui::Area::new(egui::Id::new("capture_mode_bar"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 21.0))
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(theme::PILL_BG)
                .corner_radius(egui::CornerRadius::same(23))
                .inner_margin(egui::Margin::symmetric(12, 8))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        mode_chip(ui, "Région", true)
                            .on_hover_text("Tracez un rectangle sur l'écran gelé");
                        if mode_chip(ui, "Fenêtre…", false)
                            .on_hover_text("Choisir une fenêtre à capturer")
                            .clicked()
                        {
                            *action = Some(ModeAction::PickWindow);
                        }
                        if mode_chip(ui, "Écran entier", false)
                            .on_hover_text("Capturer tout l'écran (Entrée)")
                            .clicked()
                        {
                            *committed = Some(None);
                        }
                        if mode_chip(ui, "Dans 3 s", false)
                            .on_hover_text("Région après un délai de 3 secondes")
                            .clicked()
                        {
                            *action = Some(ModeAction::RegionIn3s);
                        }
                        bar_separator(ui);
                        bar_text(ui, "Tracez un rectangle pour capturer");
                        bar_separator(ui);
                        if key_chip(ui, "Échap").on_hover_text("Annuler").clicked() {
                            *action = Some(ModeAction::Cancel);
                        }
                    });
                });
        });
}

/// Pastille de mode : pleine (accent) si active, sinon translucide au survol.
fn mode_chip(ui: &mut egui::Ui, label: &str, active: bool) -> egui::Response {
    let font = egui::FontId::proportional(14.0);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), egui::Color32::WHITE);
    let size = galley.size() + egui::vec2(24.0, 12.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let fill = if active {
        theme::ACCENT
    } else if resp.hovered() {
        egui::Color32::from_white_alpha(26)
    } else {
        egui::Color32::TRANSPARENT
    };
    let text_color = if active {
        egui::Color32::WHITE
    } else {
        theme::PILL_KEY_TEXT
    };
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(15), fill);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, text_color);
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, text_color);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Badge de touche clavier (bordure fine), cliquable.
fn key_chip(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(13.0),
        theme::PILL_KEY_TEXT,
    );
    let size = galley.size() + egui::vec2(16.0, 10.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    ui.painter().rect_stroke(
        rect,
        egui::CornerRadius::same(7),
        egui::Stroke::new(1.0, theme::PILL_KEY_BORDER),
        egui::StrokeKind::Middle,
    );
    ui.painter().galley(
        rect.center() - galley.size() / 2.0,
        galley,
        theme::PILL_KEY_TEXT,
    );
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Texte d'aide non interactif dans la barre.
fn bar_text(ui: &mut egui::Ui, label: &str) {
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(14.0),
        egui::Color32::WHITE,
    );
    let size = galley.size() + egui::vec2(8.0, 12.0);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.painter().galley(
        rect.center() - galley.size() / 2.0,
        galley,
        egui::Color32::WHITE,
    );
}

/// Séparateur vertical fin.
fn bar_separator(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 22.0), egui::Sense::hover());
    ui.painter().line_segment(
        [rect.center_top(), rect.center_bottom()],
        egui::Stroke::new(1.0, theme::PILL_KEY_BORDER),
    );
}

fn restore_window(ctx: &egui::Context) {
    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
    ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
}
