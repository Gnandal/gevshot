//! Icônes peintes au trait propres à GEVSTUDIO (~16 px), dans le style de
//! `gev_ui::widgets` : chaque fonction dessine centrée sur `c`.

use eframe::egui::{self, Color32, FontId, Painter, Pos2, Rect, Stroke, vec2};

/// Document : feuille avec lignes.
pub fn icon_doc(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    p.rect_stroke(
        Rect::from_center_size(c, vec2(11.0, 14.0)),
        egui::CornerRadius::same(2),
        stroke,
        egui::StrokeKind::Middle,
    );
    for dy in [-2.5, 0.5, 3.5] {
        p.line_segment([c + vec2(-3.0, dy), c + vec2(3.0, dy)], stroke);
    }
}

/// Slides : écran avec pied.
pub fn icon_slides(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    p.rect_stroke(
        Rect::from_center_size(c + vec2(0.0, -1.5), vec2(15.0, 10.0)),
        egui::CornerRadius::same(2),
        stroke,
        egui::StrokeKind::Middle,
    );
    p.line_segment([c + vec2(0.0, 3.5), c + vec2(0.0, 6.0)], stroke);
    p.line_segment([c + vec2(-3.5, 6.5), c + vec2(3.5, 6.5)], stroke);
}

/// Vidéo : clap de cinéma simplifié.
pub fn icon_movie(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    p.rect_stroke(
        Rect::from_center_size(c + vec2(0.0, 1.5), vec2(15.0, 10.0)),
        egui::CornerRadius::same(2),
        stroke,
        egui::StrokeKind::Middle,
    );
    p.line_segment([c + vec2(-7.5, -3.5), c + vec2(7.5, -3.5)], stroke);
    for dx in [-4.5, 0.0, 4.5] {
        p.line_segment([c + vec2(dx, -3.5), c + vec2(dx + 2.5, -6.5)], stroke);
    }
}

/// Plus.
pub fn icon_plus(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.8, color);
    p.line_segment([c + vec2(-5.5, 0.0), c + vec2(5.5, 0.0)], stroke);
    p.line_segment([c + vec2(0.0, -5.5), c + vec2(0.0, 5.5)], stroke);
}

/// Chevron vers le haut.
pub fn icon_up(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.8, color);
    p.line_segment([c + vec2(-4.5, 2.0), c + vec2(0.0, -2.5)], stroke);
    p.line_segment([c + vec2(0.0, -2.5), c + vec2(4.5, 2.0)], stroke);
}

/// Chevron vers le bas.
pub fn icon_down(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.8, color);
    p.line_segment([c + vec2(-4.5, -2.0), c + vec2(0.0, 2.5)], stroke);
    p.line_segment([c + vec2(0.0, 2.5), c + vec2(4.5, -2.0)], stroke);
}

/// Dupliquer : deux rectangles superposés.
pub fn icon_duplicate(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.5, color);
    p.rect_stroke(
        Rect::from_center_size(c + vec2(1.8, 1.8), vec2(9.5, 9.5)),
        egui::CornerRadius::same(2),
        stroke,
        egui::StrokeKind::Middle,
    );
    p.line_segment([c + vec2(-4.8, -6.6), c + vec2(3.2, -6.6)], stroke);
    p.line_segment([c + vec2(-6.6, -4.8), c + vec2(-6.6, 3.2)], stroke);
}

/// Corbeille.
pub fn icon_trash(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.5, color);
    p.line_segment([c + vec2(-6.0, -4.5), c + vec2(6.0, -4.5)], stroke);
    p.line_segment([c + vec2(-2.5, -6.5), c + vec2(2.5, -6.5)], stroke);
    let body = [
        c + vec2(-4.5, -4.5),
        c + vec2(-3.7, 6.5),
        c + vec2(3.7, 6.5),
        c + vec2(4.5, -4.5),
    ];
    p.line_segment([body[0], body[1]], stroke);
    p.line_segment([body[1], body[2]], stroke);
    p.line_segment([body[2], body[3]], stroke);
    p.line_segment([c + vec2(-1.4, -2.0), c + vec2(-1.2, 4.0)], stroke);
    p.line_segment([c + vec2(1.4, -2.0), c + vec2(1.2, 4.0)], stroke);
}

/// Import d'image : cadre + montagne + soleil.
pub fn icon_image_plus(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.5, color);
    p.rect_stroke(
        Rect::from_center_size(c + vec2(-1.0, 0.0), vec2(13.0, 11.0)),
        egui::CornerRadius::same(2),
        stroke,
        egui::StrokeKind::Middle,
    );
    p.line_segment([c + vec2(-6.0, 3.5), c + vec2(-2.5, -0.5)], stroke);
    p.line_segment([c + vec2(-2.5, -0.5), c + vec2(0.5, 2.5)], stroke);
    p.circle_stroke(c + vec2(-4.0, -2.5), 1.2, stroke);
    p.line_segment([c + vec2(5.0, -6.5), c + vec2(5.0, -1.5)], Stroke::new(1.8, color));
    p.line_segment([c + vec2(2.5, -4.0), c + vec2(7.5, -4.0)], Stroke::new(1.8, color));
}

/// Export : flèche sortant d'un bac.
pub fn icon_export(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.7, color);
    p.line_segment([c + vec2(0.0, -7.0), c + vec2(0.0, 3.0)], stroke);
    p.line_segment([c + vec2(-3.8, -3.5), c + vec2(0.0, -7.2)], stroke);
    p.line_segment([c + vec2(3.8, -3.5), c + vec2(0.0, -7.2)], stroke);
    p.line_segment([c + vec2(-6.0, 3.0), c + vec2(-6.0, 6.5)], stroke);
    p.line_segment([c + vec2(-6.0, 6.5), c + vec2(6.0, 6.5)], stroke);
    p.line_segment([c + vec2(6.0, 6.5), c + vec2(6.0, 3.0)], stroke);
}

/// Nuage avec coche (état enregistré).
pub fn icon_cloud_check(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.4, color);
    // Silhouette de nuage en arcs approximés.
    p.circle_stroke(c + vec2(-3.0, 1.0), 3.2, stroke);
    p.circle_stroke(c + vec2(1.5, -1.0), 4.0, stroke);
    p.circle_stroke(c + vec2(4.5, 2.0), 2.6, stroke);
    p.line_segment([c + vec2(-3.0, 4.2), c + vec2(4.5, 4.6)], stroke);
    // Coche.
    let check = Stroke::new(1.8, color);
    p.line_segment([c + vec2(-1.5, 1.0), c + vec2(0.2, 2.8)], check);
    p.line_segment([c + vec2(0.2, 2.8), c + vec2(3.2, -0.8)], check);
}

/// Éclair (Pro).
pub fn icon_bolt(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    let pts = [
        c + vec2(1.5, -7.0),
        c + vec2(-4.0, 1.0),
        c + vec2(-0.5, 1.0),
        c + vec2(-1.5, 7.0),
        c + vec2(4.0, -1.0),
        c + vec2(0.5, -1.0),
    ];
    p.add(egui::Shape::closed_line(pts.to_vec(), stroke));
}

/// Coche dans un cercle (statut prêt).
pub fn icon_check_circle(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.5, color);
    p.circle_stroke(c, 6.5, stroke);
    p.line_segment([c + vec2(-2.8, 0.2), c + vec2(-0.8, 2.4)], stroke);
    p.line_segment([c + vec2(-0.8, 2.4), c + vec2(3.0, -2.2)], stroke);
}

/// Horloge (durée d'une étape).
pub fn icon_clock(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.3, color);
    p.circle_stroke(c, 5.5, stroke);
    p.line_segment([c, c + vec2(0.0, -3.2)], stroke);
    p.line_segment([c, c + vec2(2.4, 1.2)], stroke);
}

/// Barres de niveau (narration).
pub fn icon_waveform(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.7, color);
    for (i, h) in [3.0, 6.5, 4.5, 7.5, 3.5f32].iter().enumerate() {
        let x = -6.0 + i as f32 * 3.0;
        p.line_segment([c + vec2(x, -h), c + vec2(x, *h)], stroke);
    }
}

/// Numéro dans un rond plein (badge d'étape).
pub fn step_badge(p: &Painter, rect: Rect, n: usize, active: bool) {
    let (bg, fg) = if active {
        (gev_ui::theme::ACCENT, Color32::WHITE)
    } else {
        (gev_ui::theme::BG_PANEL3, gev_ui::theme::TEXT_WEAK)
    };
    p.rect_filled(rect, egui::CornerRadius::same(6), bg);
    p.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        n.to_string(),
        FontId::monospace(11.0),
        fg,
    );
}
