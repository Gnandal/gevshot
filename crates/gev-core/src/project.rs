use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use ecolor::Color32;
use emath::{Pos2, Rect, pos2, vec2};
use serde::{Deserialize, Serialize};

use crate::model::{Annotation, FontChoice, Shape};

pub const EXTENSION: &str = "gev";
const VERSION: u32 = 1;

fn default_true() -> bool {
    true
}

// Le format sur disque a ses propres types, découplés d'egui, pour rester
// stable même si la représentation interne change.

#[derive(Serialize, Deserialize)]
struct PointData {
    x: f32,
    y: f32,
}

#[derive(Serialize, Deserialize)]
struct RectData {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ShapeData {
    Badge {
        center: PointData,
        number: u32,
        /// Texte de l'étape (absent des anciens fichiers).
        #[serde(default)]
        label: String,
        /// Anneau blanc (les anciens fichiers l'avaient toujours).
        #[serde(default = "default_true")]
        bordered: bool,
    },
    Arrow {
        from: PointData,
        to: PointData,
    },
    Rect {
        rect: RectData,
    },
    Ellipse {
        rect: RectData,
    },
    Text {
        pos: PointData,
        text: String,
        size: f32,
        font: String,
        bold: bool,
    },
}

#[derive(Serialize, Deserialize)]
pub(crate) struct AnnotationData {
    shape: ShapeData,
    color: [u8; 4],
    stroke_width: f32,
}

#[derive(Serialize, Deserialize)]
struct ProjectFile {
    version: u32,
    image_png_base64: String,
    annotations: Vec<AnnotationData>,
}

pub fn save(
    path: &Path,
    image: &image::RgbaImage,
    annotations: &[Annotation],
) -> Result<(), String> {
    let file = ProjectFile {
        version: VERSION,
        image_png_base64: encode_png(image)?,
        annotations: annotations.iter().map(to_data).collect(),
    };
    let json =
        serde_json::to_string(&file).map_err(|e| format!("Sérialisation impossible : {e}"))?;
    std::fs::write(path, json).map_err(|e| format!("Écriture impossible : {e}"))
}

pub fn load(path: &Path) -> Result<(image::RgbaImage, Vec<Annotation>), String> {
    let json = std::fs::read_to_string(path).map_err(|e| format!("Lecture impossible : {e}"))?;
    let file: ProjectFile =
        serde_json::from_str(&json).map_err(|e| format!("Fichier projet invalide : {e}"))?;
    if file.version > VERSION {
        return Err(format!(
            "Projet créé avec une version plus récente (v{})",
            file.version
        ));
    }
    let image = decode_png(&file.image_png_base64)?;
    let annotations = file.annotations.iter().map(from_data).collect();
    Ok((image, annotations))
}

// ----- Codec image (partagé avec le format .gevdoc) --------------------------

/// Encode l'image en PNG puis en base64 (image intégrée au JSON).
pub(crate) fn encode_png(image: &image::RgbaImage) -> Result<String, String> {
    let mut png = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .map_err(|e| format!("Encodage PNG impossible : {e}"))?;
    Ok(BASE64.encode(&png))
}

/// Décode une image PNG base64 intégrée au JSON.
pub(crate) fn decode_png(base64: &str) -> Result<image::RgbaImage, String> {
    let png = BASE64
        .decode(base64)
        .map_err(|e| format!("Image corrompue : {e}"))?;
    Ok(image::load_from_memory(&png)
        .map_err(|e| format!("Décodage de l'image impossible : {e}"))?
        .to_rgba8())
}

// ----- Conversions -----------------------------------------------------------

fn point(p: Pos2) -> PointData {
    PointData { x: p.x, y: p.y }
}

fn rect(r: Rect) -> RectData {
    RectData {
        x: r.min.x,
        y: r.min.y,
        w: r.width(),
        h: r.height(),
    }
}

pub(crate) fn to_data(ann: &Annotation) -> AnnotationData {
    let shape = match &ann.shape {
        Shape::Badge {
            center,
            number,
            label,
            bordered,
        } => ShapeData::Badge {
            center: point(*center),
            number: *number,
            label: label.clone(),
            bordered: *bordered,
        },
        Shape::Arrow { from, to } => ShapeData::Arrow {
            from: point(*from),
            to: point(*to),
        },
        Shape::Rect { rect: r } => ShapeData::Rect { rect: rect(*r) },
        Shape::Ellipse { rect: r } => ShapeData::Ellipse { rect: rect(*r) },
        Shape::Text {
            pos,
            text,
            size,
            font,
            bold,
        } => ShapeData::Text {
            pos: point(*pos),
            text: text.clone(),
            size: *size,
            font: font.key().to_string(),
            bold: *bold,
        },
    };
    AnnotationData {
        shape,
        color: ann.color.to_array(),
        stroke_width: ann.stroke_width,
    }
}

pub(crate) fn from_data(data: &AnnotationData) -> Annotation {
    let as_pos = |p: &PointData| pos2(p.x, p.y);
    let as_rect =
        |r: &RectData| Rect::from_min_size(pos2(r.x, r.y), vec2(r.w.max(0.0), r.h.max(0.0)));
    let shape = match &data.shape {
        ShapeData::Badge {
            center,
            number,
            label,
            bordered,
        } => Shape::Badge {
            center: as_pos(center),
            number: *number,
            label: label.clone(),
            bordered: *bordered,
        },
        ShapeData::Arrow { from, to } => Shape::Arrow {
            from: as_pos(from),
            to: as_pos(to),
        },
        ShapeData::Rect { rect: r } => Shape::Rect { rect: as_rect(r) },
        ShapeData::Ellipse { rect: r } => Shape::Ellipse { rect: as_rect(r) },
        ShapeData::Text {
            pos,
            text,
            size,
            font,
            bold,
        } => Shape::Text {
            pos: as_pos(pos),
            text: text.clone(),
            size: size.clamp(4.0, 400.0),
            font: FontChoice::from_key(font),
            bold: *bold,
        },
    };
    let [r, g, b, a] = data.color;
    Annotation {
        shape,
        color: Color32::from_rgba_unmultiplied(r, g, b, a),
        stroke_width: data.stroke_width.clamp(0.5, 50.0),
    }
}
