//! Export d'un `Document` en GIF animé : une « diapo » raster par étape
//! (titre + capture aplatie + légende sur fond blanc, taille fixe), enchaînées
//! en boucle. 100 % Rust, aucune dépendance externe. Le texte réutilise les
//! fontes DejaVu embarquées via `gev_core::render`.

use std::io::Cursor;

use ecolor::Color32;
use gev_core::document::{Document, Step, Transition};
use gev_core::fonts::FontSet;
use gev_core::model::FontChoice;
use gev_core::render;
use image::codecs::gif::{GifEncoder, Repeat};
use image::{Delay, Frame, Rgba, RgbaImage, imageops};

// Diapo 16:9, en pixels.
const W: u32 = 1280;
const H: u32 = 720;
const MARGIN: i64 = 48;
const TITLE_H: i64 = 72;
const CAPTION_H: i64 = 104;
const FRAME_MS: u32 = 2500; // durée d'affichage d'une étape

const TEXT: Color32 = Color32::from_rgb(0x23, 0x26, 0x2E);

/// Assemble le document en octets GIF animé.
pub fn build(doc: &Document) -> Result<Vec<u8>, String> {
    let fonts = FontSet::embedded();
    let mut buf = Vec::new();
    {
        let mut enc = GifEncoder::new(Cursor::new(&mut buf));
        enc.set_repeat(Repeat::Infinite)
            .map_err(|e| format!("GIF (boucle) : {e}"))?;
        for step in &doc.steps {
            let slide = render_slide(step, &fonts);
            let frame = Frame::from_parts(slide, 0, 0, Delay::from_numer_denom_ms(FRAME_MS, 1));
            enc.encode_frame(frame)
                .map_err(|e| format!("GIF (image) : {e}"))?;
        }
    }
    Ok(buf)
}

/// Compose une diapo de taille fixe pour une étape.
fn render_slide(step: &Step, fonts: &FontSet) -> RgbaImage {
    let mut canvas = RgbaImage::from_pixel(W, H, Rgba([255, 255, 255, 255]));

    // Capture aplatie, ajustée dans la boîte utile.
    let flat = render::flatten(&step.image, &step.annotations, fonts);
    let box_w = W as i64 - 2 * MARGIN;
    let box_top = MARGIN + TITLE_H;
    let box_h = H as i64 - 2 * MARGIN - TITLE_H - CAPTION_H;
    let (nw, nh) = fit(flat.width(), flat.height(), box_w as u32, box_h as u32);
    let resized = imageops::resize(&flat, nw, nh, imageops::FilterType::Triangle);
    let x = MARGIN + (box_w - nw as i64) / 2;
    let y = box_top + (box_h - nh as i64) / 2;
    imageops::overlay(&mut canvas, &resized, x, y);

    // Titre (gras) en haut à gauche.
    if !step.title.is_empty()
        && let Some(font) = fonts.get(FontChoice::Sans, true)
    {
        render::draw_text_topleft(
            &mut canvas,
            font,
            (MARGIN as f32, MARGIN as f32),
            34.0,
            &step.title,
            TEXT,
        );
    }

    // Légende (une ou plusieurs lignes) en bas.
    if !step.caption.is_empty()
        && let Some(font) = fonts.get(FontChoice::Sans, false)
    {
        let mut line_y = (H as i64 - CAPTION_H + 8) as f32;
        for line in step.caption.lines() {
            render::draw_text_topleft(&mut canvas, font, (MARGIN as f32, line_y), 24.0, line, TEXT);
            line_y += 30.0;
        }
    }

    canvas
}

/// Ajuste `(img)` px dans `(box)` px en conservant le rapport d'aspect.
fn fit(img_w: u32, img_h: u32, box_w: u32, box_h: u32) -> (u32, u32) {
    let (iw, ih, bw, bh) = (img_w as u64, img_h as u64, box_w as u64, box_h as u64);
    if iw * bh <= bw * ih {
        // Limité par la hauteur.
        (((iw * bh) / ih).max(1) as u32, box_h.max(1))
    } else {
        // Limité par la largeur.
        (box_w.max(1), ((ih * bw) / iw).max(1) as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gev_core::document::Document;

    #[test]
    fn build_produces_animated_gif() {
        let mut image = image::RgbaImage::new(60, 40);
        for (x, _y, px) in image.enumerate_pixels_mut() {
            *px = image::Rgba([(x * 4) as u8, 100, 160, 255]);
        }
        let step = Step {
            image,
            annotations: Vec::new(),
            title: "Étape 1".into(),
            caption: "Ligne un\nLigne deux".into(),
            narration: String::new(),
            narration_audio: None,
            duration_override_ms: None,
            transition: Transition::Cut,
        };
        let doc = Document {
            title: "Guide".into(),
            steps: vec![step.clone(), step],
            voice_id: String::new(),
        };
        let bytes = build(&doc).expect("génération GIF");
        // En-tête GIF89a.
        assert_eq!(&bytes[..6], b"GIF89a");
        assert!(bytes.len() > 100);
    }
}
