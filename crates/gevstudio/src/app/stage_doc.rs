//! Scène Document : page A4 blanche avec aperçu aplati de l'étape sélectionnée
//! (titre, légende, capture, annotations). Mode édition (M2) : canevas Editor
//! interactif. Filigrane PRO en version gratuite.

use eframe::egui;
use gev_ui::theme;

use super::StudioApp;

/// Dessine le fond pointillé de la scène (motif radial).
pub fn dotted_background(ui: &mut egui::Ui) {
    let painter = ui.painter_at(ui.available_rect_before_wrap());
    let rect = painter.clip_rect();
    let dot_size = 2.0;
    let spacing = 16.0;
    let mut y = rect.min.y;
    while y < rect.max.y {
        let mut x = rect.min.x;
        while x < rect.max.x {
            painter.circle_filled(
                egui::pos2(x, y),
                dot_size,
                theme::BORDER.gamma_multiply(0.5),
            );
            x += spacing;
        }
        y += spacing;
    }
}

impl StudioApp {
    pub(super) fn stage_doc_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if self.doc.steps.is_empty() {
            return;
        }

        // Centrage de la page A4 sur le canevas disponible.
        let _page_width = 210.0; // mm, rapport A4
        let _page_height = 297.0;
        let zoom = (self.zoom_pct as f32) / 100.0;
        let page_w_px = 210.0 * zoom; // 1 mm ≈ 1 px à 100% (approximation)
        let page_h_px = 297.0 * zoom;

        let avail = ui.available_size();
        let offset_x = (avail.x - page_w_px).max(0.0) / 2.0;
        let offset_y = (avail.y - page_h_px).max(0.0) / 2.0;

        ui.add_space(offset_y);
        ui.horizontal(|ui| {
            ui.add_space(offset_x);
            self.page_ui(ctx, ui, page_w_px, page_h_px);
        });
    }

    /// Rendu de la page A4 avec contenu de l'étape sélectionnée.
    fn page_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, w: f32, h: f32) {
        let rect = ui.allocate_space(egui::vec2(w, h)).1;
        let painter = ui.painter_at(rect);

        // Fond blanc.
        painter.rect_filled(rect, egui::CornerRadius::ZERO, egui::Color32::WHITE);

        // Bordure légère.
        painter.rect_stroke(
            rect,
            egui::CornerRadius::ZERO,
            egui::Stroke::new(0.5, theme::BORDER),
            egui::StrokeKind::Inside,
        );

        // Copier les données utiles avant de les emprunter avec texture_for.
        let (step_title, step_caption) = match self.selected_step() {
            Some(step) => (step.title.clone(), step.caption.clone()),
            None => return,
        };

        // Layout interne : marges, contenu.
        let margin = 16.0 * (w / 210.0); // Marges proportionnelles.
        let content_rect = rect.shrink(margin);

        // Titre de l'étape.
        if !step_title.is_empty() {
            let size = 14.0 * (w / 210.0);
            let title_galley = painter.layout_no_wrap(
                step_title,
                egui::FontId::proportional(size),
                theme::TEXT,
            );
            painter.galley(
                content_rect.left_top(),
                title_galley,
                theme::TEXT,
            );
        }

        // Légende sous le titre.
        if !step_caption.is_empty() {
            let caption_galley = painter.layout(
                step_caption.clone(),
                egui::FontId::proportional(11.0 * (w / 210.0)),
                theme::TEXT_WEAK,
                (content_rect.width() * 0.8).min(1000.0),
            );
            let y_offset = 40.0;
            painter.galley(
                content_rect.left_top() + egui::vec2(0.0, y_offset),
                caption_galley,
                theme::TEXT_WEAK,
            );
        }

        // Aperçu aplati de l'image annotée, s'il existe.
        if let Some(tex) = self.texture_for(ctx, self.selected) {
            let img_y = 70.0;
            let img_rect_w = content_rect.width();
            let img_rect_h = img_rect_w * 9.0 / 16.0; // Approximation 16:9.
            let img_rect = egui::Rect::from_min_size(
                content_rect.left_top() + egui::vec2(0.0, img_y),
                egui::vec2(img_rect_w, img_rect_h),
            );
            painter.image(
                tex.id(),
                img_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
            // Légende d'image « Figure i — … ».
            if !step_caption.is_empty() {
                let fig_text = format!("Figure {} — {}", self.selected + 1, step_caption);
                let fig_galley = painter.layout(
                    fig_text,
                    egui::FontId::proportional(10.0 * (w / 210.0)),
                    theme::TEXT_WEAK,
                    img_rect.width(),
                );
                painter.galley(
                    img_rect.left_bottom() + egui::vec2(0.0, 4.0),
                    fig_galley,
                    theme::TEXT_WEAK,
                );
            }
        }

        // Filigrane si version gratuite.
        if !self.is_pro() {
            painter.text(
                content_rect.right_center() + egui::vec2(-30.0, 0.0),
                egui::Align2::RIGHT_CENTER,
                "GEVSHOT STUDIO\nVERSION GRATUITE",
                egui::FontId::monospace(11.0),
                theme::TEXT_WEAK.gamma_multiply(0.25),
            );
        }

        // Pied de page « Générés avec GevShot Studio · Page i ».
        let footer = format!(
            "Généré avec GevShot Studio · Page {}",
            self.selected + 1
        );
        let footer_galley = painter.layout_no_wrap(
            footer,
            egui::FontId::proportional(9.0),
            theme::TEXT_WEAK,
        );
        painter.galley(
            content_rect.left_bottom() + egui::vec2(0.0, 6.0),
            footer_galley,
            theme::TEXT_WEAK,
        );
    }
}
