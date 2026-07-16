use eframe::egui::{
    self, Align2, Color32, FontId, Pos2, Rect, Stroke, Vec2, emath::Rot2, epaint::EllipseShape,
};

/// Toutes les coordonnées sont exprimées en pixels de l'image source,
/// indépendamment du zoom d'affichage.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    /// Pastille numérotée ; `label` est le texte de l'étape dans la légende
    /// (affiché dans le panneau latéral, pas sur l'image).
    Badge {
        center: Pos2,
        number: u32,
        label: String,
        /// Anneau blanc autour du disque.
        bordered: bool,
    },
    /// Flèche du point de départ vers la cible (tête à l'arrivée).
    Arrow { from: Pos2, to: Pos2 },
    /// Rectangle d'encadrement.
    Rect { rect: Rect },
    /// Ellipse d'encerclement, inscrite dans son rectangle englobant.
    Ellipse { rect: Rect },
    /// Étiquette de texte libre, ancrée en haut à gauche.
    Text {
        pos: Pos2,
        text: String,
        /// Taille de police en pixels image.
        size: f32,
        font: FontChoice,
        bold: bool,
    },
}

/// Famille de police d'une annotation texte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontChoice {
    Sans,
    Serif,
    Mono,
}

impl FontChoice {
    pub const ALL: [FontChoice; 3] = [FontChoice::Sans, FontChoice::Serif, FontChoice::Mono];

    pub fn label(&self) -> &'static str {
        match self {
            FontChoice::Sans => "Sans",
            FontChoice::Serif => "Serif",
            FontChoice::Mono => "Mono",
        }
    }

    /// Clé stable pour le format projet.
    pub fn key(&self) -> &'static str {
        match self {
            FontChoice::Sans => "sans",
            FontChoice::Serif => "serif",
            FontChoice::Mono => "mono",
        }
    }

    pub fn from_key(key: &str) -> Self {
        match key {
            "serif" => FontChoice::Serif,
            "mono" => FontChoice::Mono,
            _ => FontChoice::Sans,
        }
    }

    /// Famille egui correspondante (enregistrée par `fonts::install`).
    pub fn family(&self, bold: bool) -> egui::FontFamily {
        egui::FontFamily::Name(crate::infra::fonts::family_name(*self, bold).into())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Annotation {
    pub shape: Shape,
    pub color: Color32,
    pub stroke_width: f32,
}

/// Transformation entre l'espace image (pixels source) et l'espace écran.
#[derive(Clone, Copy)]
pub struct CanvasTransform {
    /// Coin haut-gauche de l'image à l'écran.
    pub origin: Pos2,
    /// Facteur d'échelle image → écran.
    pub scale: f32,
}

impl CanvasTransform {
    pub fn to_screen(self, p: Pos2) -> Pos2 {
        self.origin + p.to_vec2() * self.scale
    }

    pub fn to_image(self, p: Pos2) -> Pos2 {
        ((p - self.origin) / self.scale).to_pos2()
    }
}

/// Rayon d'une pastille, lié à l'épaisseur de trait courante pour rester
/// proportionné sur des captures de résolutions variées.
pub fn badge_radius(stroke_width: f32) -> f32 {
    10.0 + stroke_width * 3.0
}

/// Longueur de la tête de flèche en pixels image.
fn arrow_head_len(stroke_width: f32, shaft_len: f32) -> f32 {
    (10.0 + stroke_width * 3.0).min(shaft_len * 0.5)
}

impl Annotation {
    pub fn paint(&self, painter: &egui::Painter, t: &CanvasTransform) {
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
                    FontId::new(size * t.scale, font.family(*bold)),
                    self.color,
                );
            }
        }
    }

    /// Rectangle englobant en coordonnées image (pour l'indicateur de sélection).
    pub fn bounding_rect(&self) -> Rect {
        match &self.shape {
            Shape::Badge { center, .. } => {
                Rect::from_center_size(*center, Vec2::splat(badge_radius(self.stroke_width) * 2.0))
            }
            Shape::Arrow { from, to } => Rect::from_two_pos(*from, *to),
            Shape::Rect { rect } | Shape::Ellipse { rect } => *rect,
            Shape::Text {
                pos, text, size, ..
            } => {
                // Estimation : la mesure exacte nécessiterait la fonte.
                let w = (text.chars().count() as f32 * size * 0.55).max(size * 0.5);
                Rect::from_min_size(*pos, Vec2::new(w, size * 1.3))
            }
        }
    }

    /// Teste si un point (coordonnées image) touche l'annotation,
    /// avec une tolérance exprimée en pixels image.
    pub fn hit(&self, p: Pos2, tol: f32) -> bool {
        match &self.shape {
            Shape::Badge { center, .. } => {
                center.distance(p) <= badge_radius(self.stroke_width) + tol
            }
            Shape::Arrow { from, to } => {
                dist_to_segment(p, *from, *to) <= self.stroke_width / 2.0 + tol
            }
            Shape::Rect { rect } => {
                let edges = [
                    (rect.left_top(), rect.right_top()),
                    (rect.right_top(), rect.right_bottom()),
                    (rect.right_bottom(), rect.left_bottom()),
                    (rect.left_bottom(), rect.left_top()),
                ];
                let reach = self.stroke_width / 2.0 + tol;
                edges
                    .iter()
                    .any(|(a, b)| dist_to_segment(p, *a, *b) <= reach)
            }
            Shape::Ellipse { rect } => {
                let rx = rect.width() / 2.0;
                let ry = rect.height() / 2.0;
                if rx < 1.0 || ry < 1.0 {
                    return false;
                }
                let d = p - rect.center();
                let v = ((d.x / rx).powi(2) + (d.y / ry).powi(2)).sqrt();
                // Distance approchée à la courbe, ramenée en pixels.
                (v - 1.0).abs() * rx.min(ry) <= self.stroke_width / 2.0 + tol
            }
            Shape::Text { .. } => self.bounding_rect().expand(tol).contains(p),
        }
    }

    /// Déplace l'annotation entière d'un delta (coordonnées image).
    pub fn translate(&mut self, delta: Vec2) {
        match &mut self.shape {
            Shape::Badge { center, .. } => *center += delta,
            Shape::Arrow { from, to } => {
                *from += delta;
                *to += delta;
            }
            Shape::Rect { rect } | Shape::Ellipse { rect } => *rect = rect.translate(delta),
            Shape::Text { pos, .. } => *pos += delta,
        }
    }
}

fn dist_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let len_sq = ab.length_sq();
    if len_sq < f32::EPSILON {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / len_sq).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}
