//! Sélection de région sur l'écran gelé : voile, rectangle, pilule d'aide,
//! validation ou annulation.

use eframe::egui;

use super::{Capture, GevshotApp, Stage};
use crate::ui::theme;

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

                draw_hint_pill(painter, rect);

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
        restore_window(ctx);
    }
}

/// Pilule d'aide sombre, centrée en haut de l'écran pendant la sélection.
fn draw_hint_pill(painter: &egui::Painter, screen: egui::Rect) {
    use egui::{CornerRadius, FontId, Rect, Shape, Stroke, pos2, vec2};

    let galley = painter.layout_no_wrap(
        "Tracez un rectangle pour capturer une région".to_string(),
        FontId::proportional(15.0),
        egui::Color32::WHITE,
    );
    let key_galley = painter.layout_no_wrap(
        "Échap".to_string(),
        FontId::proportional(13.0),
        theme::PILL_KEY_TEXT,
    );

    let (pad, gap, icon_w, h) = (18.0, 12.0, 18.0, 46.0);
    let key_w = key_galley.size().x + 16.0;
    let w = pad + icon_w + gap + galley.size().x + gap + key_w + pad;
    let rect = Rect::from_center_size(pos2(screen.center().x, screen.min.y + 44.0), vec2(w, h));
    painter.rect_filled(rect, CornerRadius::same(23), theme::PILL_BG);

    // Icône : carré en pointillés avec une petite flèche vers le coin.
    let stroke = Stroke::new(1.5, theme::PILL_ICON);
    let ic = Rect::from_center_size(
        pos2(rect.min.x + pad + icon_w / 2.0, rect.center().y),
        vec2(15.0, 15.0),
    );
    let corners = [
        ic.left_top(),
        ic.right_top(),
        ic.right_bottom(),
        ic.left_bottom(),
        ic.left_top(),
    ];
    for s in Shape::dashed_line(&corners, stroke, 2.6, 2.2) {
        painter.add(s);
    }
    let tip = ic.center() + vec2(-2.0, -2.0);
    painter.line_segment([ic.center() + vec2(3.5, 3.5), tip], stroke);
    painter.line_segment([tip, tip + vec2(3.5, 0.0)], stroke);
    painter.line_segment([tip, tip + vec2(0.0, 3.5)], stroke);

    // Texte.
    painter.galley(
        pos2(
            rect.min.x + pad + icon_w + gap,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        egui::Color32::WHITE,
    );

    // Touche Échap.
    let key_rect = Rect::from_min_size(
        pos2(rect.max.x - pad - key_w, rect.center().y - 13.0),
        vec2(key_w, 26.0),
    );
    painter.rect_stroke(
        key_rect,
        CornerRadius::same(7),
        Stroke::new(1.0, theme::PILL_KEY_BORDER),
        egui::StrokeKind::Middle,
    );
    let key_size = key_galley.size();
    painter.galley(
        key_rect.center() - key_size / 2.0,
        key_galley,
        theme::PILL_KEY_TEXT,
    );
}

fn restore_window(ctx: &egui::Context) {
    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
    ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
}
