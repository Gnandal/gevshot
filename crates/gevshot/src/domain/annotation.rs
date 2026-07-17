//! Ré-export du modèle partagé (gev-core) et rendu écran egui des
//! annotations. L'aplatissement sur pixels vit dans `gev_core::render`.

pub use gev_core::model::*;

use eframe::egui::{self, Align2, Color32, FontId, Rect, Stroke, emath::Rot2, epaint::EllipseShape};

/// Famille egui correspondant à un choix de police
/// (enregistrée par `infra::fonts::install`).
pub fn family(font: FontChoice, bold: bool) -> egui::FontFamily {
    egui::FontFamily::Name(gev_core::fonts::family_name(font, bold).into())
}

/// Rendu écran d'une annotation sur le canevas de l'éditeur.
pub trait PaintExt {
    fn paint(&self, painter: &egui::Painter, t: &CanvasTransform);
}

impl PaintExt for Annotation {
    fn paint(&self, painter: &egui::Painter, t: &CanvasTransform) {
        let stroke = Stroke::new((self.stroke_width * t.scale).max(1.0), self.color);
        match &self.shape {
            Shape::Badge {
                center,
                number,
                bordered,
                ..
            } => {
                let c = t.to_screen(*center);
                let r = badge_radius(self.stroke_width) * t.scale;
                painter.circle_filled(c, r, self.color);
                if *bordered {
                    painter.circle_stroke(
                        c,
                        r,
                        Stroke::new((1.5 * t.scale).max(1.0), Color32::WHITE),
                    );
                }
                painter.text(
                    c,
                    Align2::CENTER_CENTER,
                    number.to_string(),
                    FontId::proportional((r * 1.2).max(8.0)),
                    Color32::WHITE,
                );
            }
            Shape::Arrow { from, to } => {
                let a = t.to_screen(*from);
                let b = t.to_screen(*to);
                painter.line_segment([a, b], stroke);
                let shaft = b - a;
                let len = shaft.length();
                if len > 1.0 {
                    let dir = shaft / len;
                    let head = arrow_head_len(self.stroke_width, len / t.scale) * t.scale;
                    for angle in [0.5, -0.5f32] {
                        let v = Rot2::from_angle(angle) * (-dir * head);
                        painter.line_segment([b, b + v], stroke);
                    }
                }
            }
            Shape::Rect { rect } => {
                let r = Rect::from_min_max(t.to_screen(rect.min), t.to_screen(rect.max));
                painter.rect_stroke(
                    r,
                    egui::CornerRadius::ZERO,
                    stroke,
                    egui::StrokeKind::Middle,
                );
            }
            Shape::Ellipse { rect } => {
                let r = Rect::from_min_max(t.to_screen(rect.min), t.to_screen(rect.max));
                painter.add(egui::Shape::Ellipse(EllipseShape {
                    center: r.center(),
                    radius: r.size() / 2.0,
                    fill: Color32::TRANSPARENT,
                    stroke,
                    angle: 0.0,
                }));
            }
            Shape::Text {
                pos,
                text,
                size,
                font,
                bold,
            } => {
                painter.text(
                    t.to_screen(*pos),
                    Align2::LEFT_TOP,
                    text,
                    FontId::new(size * t.scale, family(*font, *bold)),
                    self.color,
                );
            }
        }
    }
}
