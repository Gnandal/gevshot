//! Rendu d'un `Document` en PDF : une page A4 par étape, empilant le titre,
//! la capture aplatie (annotations via `gev_core::render`) et la légende.
//! Texte et annotations partagent les fontes DejaVu embarquées → rendu
//! identique sur toute machine.

use gev_core::document::{Document, Step};
use gev_core::fonts::FontSet;
use gev_core::model::FontChoice;
use gev_core::render;
use printpdf::{
    Color, FontId, Mm, Op, ParsedFont, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point,
    Pt, RawImage, RawImageData, RawImageFormat, Rgb, TextItem, XObjectTransform,
};

// Géométrie de page, en millimètres.
const PAGE_W: f32 = 210.0;
const PAGE_H: f32 = 297.0;
const MARGIN: f32 = 15.0;
const CONTENT_W: f32 = PAGE_W - 2.0 * MARGIN;

const TITLE_PT: f32 = 16.0;
const CAPTION_PT: f32 = 11.0;
const GAP: f32 = 5.0; // espace vertical entre blocs

fn black() -> Color {
    Color::Rgb(Rgb {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        icc_profile: None,
    })
}

fn pt_to_mm(pt: f32) -> f32 {
    pt * 25.4 / 72.0
}

/// Assemble le document en octets PDF.
pub fn build(doc: &Document) -> Result<Vec<u8>, String> {
    let fonts = FontSet::embedded();
    let regular_bytes = fonts
        .bytes(FontChoice::Sans, false)
        .ok_or("fonte sans embarquée manquante")?;
    let bold_bytes = fonts
        .bytes(FontChoice::Sans, true)
        .ok_or("fonte sans grasse embarquée manquante")?;

    let mut pdf = PdfDocument::new(if doc.title.is_empty() {
        "GEVSTUDIO"
    } else {
        &doc.title
    });

    let regular = parse_font(regular_bytes)?;
    let bold = parse_font(bold_bytes)?;
    let regular_id = pdf.add_font(&regular);
    let bold_id = pdf.add_font(&bold);

    let mut pages = Vec::with_capacity(doc.steps.len());
    for step in &doc.steps {
        pages.push(build_page(&mut pdf, step, &fonts, &regular_id, &bold_id));
    }

    let bytes = pdf.with_pages(pages).save(
        &PdfSaveOptions {
            subset_fonts: true,
            ..Default::default()
        },
        &mut Vec::new(),
    );
    Ok(bytes)
}

fn parse_font(bytes: &[u8]) -> Result<ParsedFont, String> {
    ParsedFont::from_bytes(bytes, 0, &mut Vec::new()).ok_or_else(|| "fonte illisible".to_string())
}

fn build_page(
    pdf: &mut PdfDocument,
    step: &Step,
    fonts: &FontSet,
    regular: &FontId,
    bold: &FontId,
) -> PdfPage {
    // Aplatit les annotations sur la capture (le `.gevd` les garde séparées
    // pour rester éditable ; le PDF, lui, est figé).
    let flat = render::flatten(&step.image, &step.annotations, fonts);

    let mut ops = Vec::new();
    // Descend depuis le haut de la zone utile.
    let mut cursor_y = PAGE_H - MARGIN;

    if !step.title.is_empty() {
        let baseline = cursor_y - pt_to_mm(TITLE_PT);
        push_text(&mut ops, &step.title, bold, TITLE_PT, MARGIN, baseline);
        cursor_y = baseline - GAP;
    }

    // Hauteur restante pour l'image : on réserve la place de la légende.
    let caption_lines = if step.caption.is_empty() {
        0
    } else {
        step.caption.lines().count().max(1)
    };
    let caption_block = if caption_lines == 0 {
        0.0
    } else {
        GAP + caption_lines as f32 * pt_to_mm(CAPTION_PT) * 1.35
    };
    let max_img_h = (cursor_y - MARGIN - caption_block).max(1.0);

    let (w_mm, h_mm) = fit(
        flat.width() as f32,
        flat.height() as f32,
        CONTENT_W,
        max_img_h,
    );
    let image_id = pdf.add_image(&raw_image(&flat));
    let x_mm = MARGIN + (CONTENT_W - w_mm) / 2.0; // centré horizontalement
    let img_bottom = cursor_y - h_mm;
    // dpi tel que la largeur en pixels rende exactement w_mm.
    let dpi = flat.width() as f32 * 25.4 / w_mm;
    ops.push(Op::UseXobject {
        id: image_id,
        transform: XObjectTransform {
            translate_x: Some(Mm(x_mm).into_pt()),
            translate_y: Some(Mm(img_bottom).into_pt()),
            rotate: None,
            scale_x: Some(1.0),
            scale_y: Some(1.0),
            dpi: Some(dpi),
        },
    });
    cursor_y = img_bottom - GAP;

    if caption_lines > 0 {
        let line_mm = pt_to_mm(CAPTION_PT) * 1.35;
        let mut baseline = cursor_y - pt_to_mm(CAPTION_PT);
        for line in step.caption.lines() {
            push_text(&mut ops, line, regular, CAPTION_PT, MARGIN, baseline);
            baseline -= line_mm;
        }
    }

    PdfPage::new(Mm(PAGE_W), Mm(PAGE_H), ops)
}

/// Ajuste `(img_w, img_h)` pixels dans une boîte `(box_w, box_h)` mm en
/// conservant le rapport d'aspect. Ne pas agrandir au-delà de la largeur.
fn fit(img_w: f32, img_h: f32, box_w: f32, box_h: f32) -> (f32, f32) {
    let aspect = img_h / img_w;
    let mut w = box_w;
    let mut h = w * aspect;
    if h > box_h {
        h = box_h;
        w = h / aspect;
    }
    (w, h)
}

fn raw_image(img: &image::RgbaImage) -> RawImage {
    RawImage {
        width: img.width() as usize,
        height: img.height() as usize,
        pixels: RawImageData::U8(img.as_raw().clone()),
        data_format: RawImageFormat::RGBA8,
        tag: Vec::new(),
    }
}

fn push_text(ops: &mut Vec<Op>, text: &str, font: &FontId, size: f32, x_mm: f32, y_mm: f32) {
    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor {
        pos: Point::new(Mm(x_mm), Mm(y_mm)),
    });
    ops.push(Op::SetFont {
        font: PdfFontHandle::External(font.clone()),
        size: Pt(size),
    });
    ops.push(Op::SetFillColor { col: black() });
    ops.push(Op::ShowText {
        items: vec![TextItem::Text(text.to_string())],
    });
    ops.push(Op::EndTextSection);
}

#[cfg(test)]
mod tests {
    use super::*;
    use gev_core::model::{Annotation, Shape};

    fn sample_step(title: &str, caption: &str) -> Step {
        let mut image = image::RgbaImage::new(80, 50);
        for (x, _y, px) in image.enumerate_pixels_mut() {
            *px = image::Rgba([(x * 3) as u8, 120, 200, 255]);
        }
        let annotations = vec![Annotation {
            shape: Shape::Rect {
                rect: emath::Rect::from_min_size(emath::pos2(10.0, 10.0), emath::vec2(30.0, 20.0)),
            },
            color: ecolor::Color32::RED,
            stroke_width: 2.0,
        }];
        Step {
            image,
            annotations,
            title: title.into(),
            caption: caption.into(),
        }
    }

    #[test]
    fn build_produces_valid_pdf() {
        let doc = Document {
            title: "Test".into(),
            steps: vec![
                sample_step("Étape 1", "Une légende accentuée."),
                sample_step("", ""),
            ],
        };
        let bytes = build(&doc).expect("génération PDF");
        assert!(bytes.starts_with(b"%PDF"), "en-tête PDF manquant");
        assert!(bytes.len() > 1000, "PDF suspicieusement petit");
    }
}
