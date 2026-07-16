use std::borrow::Cow;
use std::path::{Path, PathBuf};

use ab_glyph::{Font, FontVec, ScaleFont};
use eframe::egui::Color32;
use tiny_skia::{FillRule, LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::annotation::{Annotation, FontChoice, Shape, badge_radius};

// ----- Aplatissement des annotations sur les pixels -------------------------

/// Rend les annotations sur une copie de l'image source et la retourne.
pub fn flatten(image: &image::RgbaImage, annotations: &[Annotation]) -> image::RgbaImage {
    let (w, h) = (image.width(), image.height());
    let Some(size) = tiny_skia::IntSize::from_wh(w, h) else {
        return image.clone();
    };
    // L'image capturée est opaque : RGBA « droit » et prémultiplié coïncident.
    let Some(mut pixmap) = Pixmap::from_vec(image.as_raw().clone(), size) else {
        return image.clone();
    };

    for ann in annotations {
        draw_annotation(&mut pixmap, ann);
    }

    let mut out = image::RgbaImage::from_raw(w, h, pixmap.take()).unwrap_or_else(|| image.clone());

    // Le texte est rendu après coup (tiny-skia ne fait pas de texte).
    for ann in annotations {
        match &ann.shape {
            Shape::Badge { center, number, .. } => {
                let font = crate::fonts::export_font(FontChoice::Sans, true)
                    .or_else(|| crate::fonts::export_font(FontChoice::Sans, false));
                let Some(font) = font else { continue };
                let r = badge_radius(ann.stroke_width);
                draw_text_centered(
                    &mut out,
                    font,
                    (center.x, center.y),
                    (r * 1.2).max(8.0),
                    &number.to_string(),
                    Color32::WHITE,
                );
            }
            Shape::Text {
                pos,
                text,
                size,
                font,
                bold,
            } => {
                let Some(font) = crate::fonts::export_font(*font, *bold) else {
                    continue;
                };
                draw_text_topleft(&mut out, font, (pos.x, pos.y), *size, text, ann.color);
            }
            _ => {}
        }
    }
    out
}

fn draw_annotation(pixmap: &mut Pixmap, ann: &Annotation) {
    let mut paint = Paint {
        anti_alias: true,
        ..Paint::default()
    };
    paint.set_color_rgba8(ann.color.r(), ann.color.g(), ann.color.b(), 255);
    let stroke = Stroke {
        width: ann.stroke_width,
        line_cap: LineCap::Round,
        ..Stroke::default()
    };
    let identity = Transform::identity();

    match &ann.shape {
        Shape::Badge {
            center, bordered, ..
        } => {
            let r = badge_radius(ann.stroke_width);
            let mut pb = PathBuilder::new();
            pb.push_circle(center.x, center.y, r);
            if let Some(path) = pb.finish() {
                pixmap.fill_path(&path, &paint, FillRule::Winding, identity, None);
                if *bordered {
                    let mut white = Paint {
                        anti_alias: true,
                        ..Paint::default()
                    };
                    white.set_color_rgba8(255, 255, 255, 255);
                    let ring = Stroke {
                        width: 1.5,
                        ..Stroke::default()
                    };
                    pixmap.stroke_path(&path, &white, &ring, identity, None);
                }
            }
        }
        Shape::Arrow { from, to } => {
            let mut pb = PathBuilder::new();
            pb.move_to(from.x, from.y);
            pb.line_to(to.x, to.y);
            let shaft = *to - *from;
            let len = shaft.length();
            if len > 1.0 {
                let dir = shaft / len;
                let head = (10.0 + ann.stroke_width * 3.0).min(len * 0.5);
                for angle in [0.5, -0.5f32] {
                    let v = eframe::egui::emath::Rot2::from_angle(angle) * (-dir * head);
                    pb.move_to(to.x, to.y);
                    pb.line_to(to.x + v.x, to.y + v.y);
                }
            }
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, identity, None);
            }
        }
        Shape::Rect { rect } => {
            if let Some(r) =
                tiny_skia::Rect::from_ltrb(rect.min.x, rect.min.y, rect.max.x, rect.max.y)
            {
                let mut pb = PathBuilder::new();
                pb.push_rect(r);
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, &paint, &stroke, identity, None);
                }
            }
        }
        Shape::Ellipse { rect } => {
            if let Some(r) =
                tiny_skia::Rect::from_ltrb(rect.min.x, rect.min.y, rect.max.x, rect.max.y)
            {
                let mut pb = PathBuilder::new();
                pb.push_oval(r);
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, &paint, &stroke, identity, None);
                }
            }
        }
        // Rendu séparément, après le passage tiny-skia.
        Shape::Text { .. } => {}
    }
}

// ----- Texte (pastilles et étiquettes) ---------------------------------------

/// Mélange un pixel avec `color` selon une couverture [0..1].
fn blend_px(img: &mut image::RgbaImage, x: i32, y: i32, color: Color32, coverage: f32) {
    if x < 0 || y < 0 || x >= img.width() as i32 || y >= img.height() as i32 || coverage <= 0.0 {
        return;
    }
    let dst = img.get_pixel_mut(x as u32, y as u32);
    let a = coverage.clamp(0.0, 1.0);
    for c in 0..3 {
        let src = [color.r(), color.g(), color.b()][c] as f32;
        dst.0[c] = (src * a + dst.0[c] as f32 * (1.0 - a)).round() as u8;
    }
}

/// Dessine `text` ancré en haut à gauche sur `pos`, comme le rendu egui.
fn draw_text_topleft(
    img: &mut image::RgbaImage,
    font: &FontVec,
    pos: (f32, f32),
    px: f32,
    text: &str,
    color: Color32,
) {
    let scaled = font.as_scaled(ab_glyph::PxScale::from(px));
    let baseline = pos.1 + scaled.ascent();
    let mut x = pos.0;
    for ch in text.chars() {
        let mut glyph = scaled.scaled_glyph(ch);
        glyph.position = ab_glyph::point(x, baseline);
        x += scaled.h_advance(glyph.id);
        if let Some(og) = font.outline_glyph(glyph) {
            let b = og.px_bounds();
            og.draw(|gx, gy, coverage| {
                blend_px(
                    img,
                    (b.min.x + gx as f32).round() as i32,
                    (b.min.y + gy as f32).round() as i32,
                    color,
                    coverage,
                );
            });
        }
    }
}

/// Dessine `text` centré (horizontalement et verticalement) sur `center`.
fn draw_text_centered(
    img: &mut image::RgbaImage,
    font: &FontVec,
    center: (f32, f32),
    px: f32,
    text: &str,
    color: Color32,
) {
    let scaled = font.as_scaled(ab_glyph::PxScale::from(px));

    // Première passe : glyphes posés sur une ligne de base arbitraire pour
    // mesurer le rectangle englobant réel.
    let mut glyphs = Vec::new();
    let mut x = 0.0f32;
    for ch in text.chars() {
        let mut glyph = scaled.scaled_glyph(ch);
        glyph.position = ab_glyph::point(x, 0.0);
        x += scaled.h_advance(glyph.id);
        glyphs.push(glyph);
    }
    let mut min = (f32::MAX, f32::MAX);
    let mut max = (f32::MIN, f32::MIN);
    let outlined: Vec<_> = glyphs
        .into_iter()
        .filter_map(|g| font.outline_glyph(g))
        .collect();
    for og in &outlined {
        let b = og.px_bounds();
        min.0 = min.0.min(b.min.x);
        min.1 = min.1.min(b.min.y);
        max.0 = max.0.max(b.max.x);
        max.1 = max.1.max(b.max.y);
    }
    if outlined.is_empty() || min.0 > max.0 {
        return;
    }

    // Décalage pour amener le centre du rectangle englobant sur `center`.
    let offset = (
        center.0 - (min.0 + max.0) / 2.0,
        center.1 - (min.1 + max.1) / 2.0,
    );
    let (iw, ih) = (img.width() as i32, img.height() as i32);
    for og in &outlined {
        let b = og.px_bounds();
        og.draw(|gx, gy, coverage| {
            let px_x = (b.min.x + gx as f32 + offset.0).round() as i32;
            let px_y = (b.min.y + gy as f32 + offset.1).round() as i32;
            if px_x < 0 || px_y < 0 || px_x >= iw || px_y >= ih || coverage <= 0.0 {
                return;
            }
            let dst = img.get_pixel_mut(px_x as u32, px_y as u32);
            let a = coverage.clamp(0.0, 1.0);
            for c in 0..3 {
                let src = [color.r(), color.g(), color.b()][c] as f32;
                dst.0[c] = (src * a + dst.0[c] as f32 * (1.0 - a)).round() as u8;
            }
        });
    }
}

// ----- Sorties ---------------------------------------------------------------

/// Copie du texte dans le presse-papiers.
pub fn copy_text(text: &str) -> Result<(), String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("Presse-papiers indisponible : {e}"))?;
    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("Copie impossible : {e}"))
}

/// Copie l'image dans le presse-papiers.
pub fn copy_to_clipboard(img: &image::RgbaImage) -> Result<(), String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("Presse-papiers indisponible : {e}"))?;
    clipboard
        .set_image(arboard::ImageData {
            width: img.width() as usize,
            height: img.height() as usize,
            bytes: Cow::Borrowed(img.as_raw()),
        })
        .map_err(|e| format!("Copie impossible : {e}"))
}

/// Dossier de sauvegarde automatique par défaut : Images\Gevshot.
pub fn default_save_dir() -> PathBuf {
    dirs::picture_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Gevshot")
}

/// Prochain nom auto-incrémenté (sans extension) : `capture-AAAA-MM-JJ-NNN`.
pub fn next_auto_name(dir: &Path) -> String {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let prefix = format!("capture-{date}-");
    let next = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            name.strip_prefix(&prefix)?
                .strip_suffix(".png")?
                .parse::<u32>()
                .ok()
        })
        .max()
        .unwrap_or(0)
        + 1;
    format!("{prefix}{next:03}")
}

/// Sauvegarde en PNG sous un nom auto-incrémenté dans `dir`.
pub fn save_auto(img: &image::RgbaImage, dir: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("Création du dossier impossible : {e}"))?;
    let path = dir.join(format!("{}.png", next_auto_name(dir)));
    img.save(&path)
        .map_err(|e| format!("Écriture impossible : {e}"))?;
    Ok(path)
}

/// Sauvegarde via une boîte de dialogue « Enregistrer sous ».
pub fn save_dialog(img: &image::RgbaImage, dir: &Path) -> Result<Option<PathBuf>, String> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Image PNG", &["png"])
        .set_file_name(format!("{}.png", next_auto_name(dir)))
        .set_directory(dir)
        .save_file()
    else {
        return Ok(None);
    };
    img.save(&path)
        .map_err(|e| format!("Écriture impossible : {e}"))?;
    Ok(Some(path))
}
