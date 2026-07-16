use eframe::egui::{
    self, Align2, Color32, FontId, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, vec2,
};

use crate::theme;

/// Bouton d'outil : icône peinte au trait, fond bleu clair arrondi si actif.
pub fn icon_button(
    ui: &mut Ui,
    selected: bool,
    draw: impl FnOnce(&Painter, Pos2, Color32),
) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(34.0, 30.0), Sense::click());
    if selected {
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(8), theme::ACCENT_SOFT);
    } else if resp.hovered() {
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(8), theme::BG_HOVER);
    }
    let color = if selected { theme::ACCENT } else { theme::TEXT };
    draw(ui.painter(), rect.center(), color);
    resp
}

/// Variante désactivable du bouton d'icône (grisé, non cliquable).
pub fn icon_button_enabled(
    ui: &mut Ui,
    enabled: bool,
    draw: impl FnOnce(&Painter, Pos2, Color32),
) -> Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(vec2(34.0, 30.0), sense);
    if enabled && resp.hovered() {
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(8), theme::BG_HOVER);
    }
    let color = if enabled {
        theme::TEXT
    } else {
        Color32::from_rgb(0xB6, 0xB4, 0xC2)
    };
    draw(ui.painter(), rect.center(), color);
    resp
}

/// Pastille de couleur ronde ; anneau de la même couleur si sélectionnée.
pub fn swatch(ui: &mut Ui, color: Color32, selected: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(26.0, 26.0), Sense::click());
    let c = rect.center();
    ui.painter().circle_filled(c, 9.0, color);
    if selected {
        ui.painter().circle_stroke(c, 12.0, Stroke::new(2.0, color));
    } else if resp.hovered() {
        ui.painter()
            .circle_stroke(c, 11.5, Stroke::new(1.0, theme::BORDER));
    }
    resp
}

/// Sélecteur d'épaisseur : un point dont la taille reflète le trait.
pub fn stroke_dot(ui: &mut Ui, stroke_width: f32, selected: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(30.0, 28.0), Sense::click());
    if selected {
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(8), theme::ACCENT_SOFT);
    } else if resp.hovered() {
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(8), theme::BG_HOVER);
    }
    let color = if selected {
        theme::ACCENT
    } else {
        Color32::from_rgb(0x3C, 0x40, 0x4A)
    };
    ui.painter()
        .circle_filled(rect.center(), 1.5 + stroke_width * 0.75, color);
    resp
}

/// Bouton pilule : bleu plein (primaire) ou bleu clair (secondaire).
pub fn pill_button(ui: &mut Ui, text: &str, primary: bool) -> Response {
    let (fill, text_color) = if primary {
        (theme::ACCENT, Color32::WHITE)
    } else {
        (theme::ACCENT_SOFT, theme::ACCENT)
    };
    ui.add(
        egui::Button::new(egui::RichText::new(text).color(text_color).strong())
            .fill(fill)
            .corner_radius(egui::CornerRadius::same(16))
            .min_size(vec2(92.0, 32.0)),
    )
}

// ----- Icônes d'outils (dessinées au trait, ~16 px) -------------------------

pub fn icon_select(p: &Painter, c: Pos2, color: Color32) {
    let base = c + vec2(-3.5, -7.5);
    let pts: Vec<Pos2> = [
        (0.0, 0.0),
        (0.0, 13.0),
        (3.2, 9.8),
        (5.6, 14.6),
        (7.8, 13.6),
        (5.6, 9.0),
        (10.0, 9.0),
    ]
    .iter()
    .map(|&(x, y)| base + vec2(x, y))
    .collect();
    p.add(egui::Shape::closed_line(pts, Stroke::new(1.6, color)));
}

pub fn icon_badge(p: &Painter, c: Pos2, color: Color32) {
    p.circle_stroke(c, 7.5, Stroke::new(1.6, color));
    p.text(
        c,
        Align2::CENTER_CENTER,
        "1",
        FontId::proportional(10.0),
        color,
    );
}

pub fn icon_arrow(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.8, color);
    let a = c + vec2(-6.0, 6.0);
    let b = c + vec2(6.0, -6.0);
    p.line_segment([a, b], stroke);
    p.line_segment([b, b + vec2(-6.0, 0.5)], stroke);
    p.line_segment([b, b + vec2(-0.5, 6.0)], stroke);
}

pub fn icon_rect(p: &Painter, c: Pos2, color: Color32) {
    p.rect_stroke(
        Rect::from_center_size(c, vec2(15.0, 12.0)),
        egui::CornerRadius::same(2),
        Stroke::new(1.6, color),
        egui::StrokeKind::Middle,
    );
}

/// Étincelle (fonctions IA) : étoile à quatre branches + petit éclat.
pub fn icon_sparkle(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.5, color);
    let big = c + vec2(-1.5, 1.0);
    let pts: Vec<Pos2> = [
        (0.0, -7.0),
        (1.8, -1.8),
        (7.0, 0.0),
        (1.8, 1.8),
        (0.0, 7.0),
        (-1.8, 1.8),
        (-7.0, 0.0),
        (-1.8, -1.8),
    ]
    .iter()
    .map(|&(x, y)| big + vec2(x, y))
    .collect();
    p.add(egui::Shape::closed_line(pts, stroke));
    let small = c + vec2(6.0, -6.0);
    p.line_segment([small + vec2(0.0, -2.5), small + vec2(0.0, 2.5)], stroke);
    p.line_segment([small + vec2(-2.5, 0.0), small + vec2(2.5, 0.0)], stroke);
}

pub fn icon_gear(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    p.circle_stroke(c, 4.6, stroke);
    p.circle_stroke(c, 1.6, stroke);
    for i in 0..8 {
        let a = i as f32 * std::f32::consts::TAU / 8.0;
        let dir = vec2(a.cos(), a.sin());
        p.line_segment([c + dir * 5.4, c + dir * 7.8], stroke);
    }
}

/// Trois lignes horizontales (panneau légende).
pub fn icon_menu_lines(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    for dy in [-5.0, 0.0, 5.0] {
        p.line_segment([c + vec2(-6.5, dy), c + vec2(6.5, dy)], stroke);
    }
}

/// Chevron vers le bas (menu des modes de capture).
pub fn icon_chevron_down(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.8, color);
    p.line_segment([c + vec2(-4.5, -2.0), c + vec2(0.0, 2.5)], stroke);
    p.line_segment([c + vec2(0.0, 2.5), c + vec2(4.5, -2.0)], stroke);
}

/// Trois points (menu de débordement).
pub fn icon_dots(p: &Painter, c: Pos2, color: Color32) {
    for dx in [-5.5, 0.0, 5.5] {
        p.circle_filled(c + vec2(dx, 0.0), 1.7, color);
    }
}

/// Croix de fermeture.
pub fn icon_close(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    p.line_segment([c + vec2(-4.5, -4.5), c + vec2(4.5, 4.5)], stroke);
    p.line_segment([c + vec2(-4.5, 4.5), c + vec2(4.5, -4.5)], stroke);
}

/// Flèche en arc (annuler / rétablir selon le sens).
fn arrow_arc(p: &Painter, c: Pos2, color: Color32, mirrored: bool) {
    let stroke = Stroke::new(1.6, color);
    let n = 16;
    let (start, end) = (0.5f32, -3.6f32);
    let pts: Vec<Pos2> = (0..=n)
        .map(|i| {
            let mut a = start + (end - start) * i as f32 / n as f32;
            if mirrored {
                a = std::f32::consts::PI - a;
            }
            c + vec2(a.cos(), a.sin()) * 6.0
        })
        .collect();
    p.add(egui::Shape::line(pts.clone(), stroke));
    let dir = (pts[n] - pts[n - 1]).normalized();
    for angle in [2.6, -2.6f32] {
        let v = egui::emath::Rot2::from_angle(angle) * (dir * 4.5);
        p.line_segment([pts[n], pts[n] + v], stroke);
    }
}

pub fn icon_undo(p: &Painter, c: Pos2, color: Color32) {
    arrow_arc(p, c, color, false);
}

pub fn icon_redo(p: &Painter, c: Pos2, color: Color32) {
    arrow_arc(p, c, color, true);
}

pub fn icon_text(p: &Painter, c: Pos2, color: Color32) {
    p.text(
        c,
        Align2::CENTER_CENTER,
        "T",
        FontId::proportional(15.0),
        color,
    );
}

pub fn icon_camera(p: &Painter, c: Pos2, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    p.rect_stroke(
        Rect::from_center_size(c + vec2(0.0, 1.0), vec2(16.0, 11.0)),
        egui::CornerRadius::same(3),
        stroke,
        egui::StrokeKind::Middle,
    );
    p.circle_stroke(c + vec2(0.0, 1.5), 3.0, stroke);
    p.line_segment([c + vec2(-3.0, -4.6), c + vec2(3.0, -4.6)], stroke);
}

pub fn icon_ellipse(p: &Painter, c: Pos2, color: Color32) {
    p.add(egui::Shape::Ellipse(egui::epaint::EllipseShape {
        center: c,
        radius: vec2(7.5, 6.0),
        fill: Color32::TRANSPARENT,
        stroke: Stroke::new(1.6, color),
        angle: 0.0,
    }));
}
