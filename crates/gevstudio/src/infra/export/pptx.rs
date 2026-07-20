//! Export d'un `Document` en présentation PowerPoint `.pptx`.
//!
//! Un `.pptx` est un paquet OPC (ZIP) de parties XML. On produit le squelette
//! minimal accepté par PowerPoint et LibreOffice : content-types, relations,
//! un thème, un masque et une mise en page, puis une diapo par étape (titre,
//! capture aplatie, légende). Les captures partagent les fontes DejaVu
//! embarquées via `gev_core::render`.

use std::io::{Cursor, Write};

use gev_core::document::{Document, Step};
use gev_core::fonts::FontSet;
use gev_core::render;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

// Diapo 16:9, en EMU (914400 EMU = 1 pouce).
const SLIDE_W: i64 = 12_192_000;
const SLIDE_H: i64 = 6_858_000;
const MARGIN: i64 = 457_200; // 0,5 pouce
const TITLE_H: i64 = 685_800; // 0,75 pouce réservé en haut
const CAPTION_H: i64 = 822_960; // ~0,9 pouce réservé en bas

/// Assemble le document en octets `.pptx`.
pub fn build(doc: &Document) -> Result<Vec<u8>, String> {
    let fonts = FontSet::embedded();
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let xml = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    let n = doc.steps.len();

    write_part(&mut zip, xml, "[Content_Types].xml", &content_types(n))?;
    write_part(&mut zip, xml, "_rels/.rels", ROOT_RELS)?;
    write_part(&mut zip, xml, "ppt/presentation.xml", &presentation(n))?;
    write_part(
        &mut zip,
        xml,
        "ppt/_rels/presentation.xml.rels",
        &presentation_rels(n),
    )?;
    write_part(&mut zip, xml, "ppt/theme/theme1.xml", THEME)?;
    write_part(
        &mut zip,
        xml,
        "ppt/slideMasters/slideMaster1.xml",
        SLIDE_MASTER,
    )?;
    write_part(
        &mut zip,
        xml,
        "ppt/slideMasters/_rels/slideMaster1.xml.rels",
        SLIDE_MASTER_RELS,
    )?;
    write_part(
        &mut zip,
        xml,
        "ppt/slideLayouts/slideLayout1.xml",
        SLIDE_LAYOUT,
    )?;
    write_part(
        &mut zip,
        xml,
        "ppt/slideLayouts/_rels/slideLayout1.xml.rels",
        SLIDE_LAYOUT_RELS,
    )?;

    for (i, step) in doc.steps.iter().enumerate() {
        let idx = i + 1;
        let flat = render::flatten(&step.image, &step.annotations, &fonts);
        let png = encode_png(&flat)?;
        write_bytes(
            &mut zip,
            stored,
            &format!("ppt/media/image{idx}.png"),
            &png,
        )?;
        write_part(
            &mut zip,
            xml,
            &format!("ppt/slides/slide{idx}.xml"),
            &slide(step, flat.width(), flat.height()),
        )?;
        write_part(
            &mut zip,
            xml,
            &format!("ppt/slides/_rels/slide{idx}.xml.rels"),
            &slide_rels(idx),
        )?;
    }

    let cursor = zip.finish().map_err(|e| format!("ZIP : {e}"))?;
    Ok(cursor.into_inner())
}

fn write_part<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    opts: SimpleFileOptions,
    name: &str,
    body: &str,
) -> Result<(), String> {
    write_bytes(zip, opts, name, body.as_bytes())
}

fn write_bytes<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    opts: SimpleFileOptions,
    name: &str,
    body: &[u8],
) -> Result<(), String> {
    zip.start_file(name, opts)
        .map_err(|e| format!("ZIP ({name}) : {e}"))?;
    zip.write_all(body)
        .map_err(|e| format!("écriture {name} : {e}"))
}

fn encode_png(img: &image::RgbaImage) -> Result<Vec<u8>, String> {
    let mut png = Vec::new();
    img.write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
        .map_err(|e| format!("Encodage PNG : {e}"))?;
    Ok(png)
}

// ----- Parties dynamiques ----------------------------------------------------

fn content_types(n: usize) -> String {
    let mut s = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Default Extension="png" ContentType="image/png"/>
<Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
<Override PartName="/ppt/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
<Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
<Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
"#,
    );
    for i in 1..=n {
        s.push_str(&format!(
            "<Override PartName=\"/ppt/slides/slide{i}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.slide+xml\"/>\n"
        ));
    }
    s.push_str("</Types>");
    s
}

const ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#;

fn presentation(n: usize) -> String {
    let mut ids = String::new();
    for i in 1..=n {
        // r:id des diapos = rId2.. ; id logique ≥ 256.
        ids.push_str(&format!(
            "<p:sldId id=\"{}\" r:id=\"rId{}\"/>",
            255 + i,
            1 + i
        ));
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rId1"/></p:sldMasterIdLst>
<p:sldIdLst>{ids}</p:sldIdLst>
<p:sldSz cx="{SLIDE_W}" cy="{SLIDE_H}"/>
<p:notesSz cx="{SLIDE_H}" cy="{SLIDE_W}"/>
</p:presentation>"#
    )
}

fn presentation_rels(n: usize) -> String {
    let mut s = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>
"#,
    );
    for i in 1..=n {
        s.push_str(&format!(
            "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide\" Target=\"slides/slide{i}.xml\"/>\n",
            1 + i
        ));
    }
    // Le thème du niveau présentation (référencé après les diapos).
    s.push_str(&format!(
        "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme\" Target=\"theme/theme1.xml\"/>\n",
        2 + n
    ));
    s.push_str("</Relationships>");
    s
}

fn slide(step: &Step, img_w: u32, img_h: u32) -> String {
    // Boîte utile pour l'image.
    let box_x = MARGIN;
    let box_y = MARGIN + TITLE_H;
    let box_w = SLIDE_W - 2 * MARGIN;
    let box_h = SLIDE_H - 2 * MARGIN - TITLE_H - CAPTION_H;
    let (w, h) = fit_emu(img_w as i64, img_h as i64, box_w, box_h);
    let x = box_x + (box_w - w) / 2;
    let y = box_y + (box_h - h) / 2;

    let title = text_box(
        "Titre",
        Emu {
            x: MARGIN,
            y: MARGIN,
            cx: SLIDE_W - 2 * MARGIN,
            cy: TITLE_H,
        },
        &escape(&step.title),
        2000,
        true,
    );
    let caption = text_box(
        "Légende",
        Emu {
            x: MARGIN,
            y: SLIDE_H - MARGIN - CAPTION_H,
            cx: SLIDE_W - 2 * MARGIN,
            cy: CAPTION_H,
        },
        &escape(&step.caption),
        1400,
        false,
    );

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld><p:spTree>
<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
<p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
{title}
<p:pic>
<p:nvPicPr><p:cNvPr id="10" name="Capture"/><p:cNvPicPr><a:picLocks noChangeAspect="1"/></p:cNvPicPr><p:nvPr/></p:nvPicPr>
<p:blipFill><a:blip r:embed="rId2"/><a:stretch><a:fillRect/></a:stretch></p:blipFill>
<p:spPr><a:xfrm><a:off x="{x}" y="{y}"/><a:ext cx="{w}" cy="{h}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>
</p:pic>
{caption}
</p:spTree></p:cSld>
<p:clrMapOvr><a:overrideClrMapping bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/></p:clrMapOvr>
</p:sld>"#
    )
}

/// Rectangle en EMU (position + taille).
struct Emu {
    x: i64,
    y: i64,
    cx: i64,
    cy: i64,
}

/// Zone de texte simple ancrée en haut, une couleur, taille en centièmes de pt.
fn text_box(name: &str, area: Emu, text: &str, sz: i32, bold: bool) -> String {
    let Emu { x, y, cx, cy } = area;
    let b = if bold { "1" } else { "0" };
    format!(
        r#"<p:sp>
<p:nvSpPr><p:cNvPr id="{name_id}" name="{name}"/><p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr><p:nvPr/></p:nvSpPr>
<p:spPr><a:xfrm><a:off x="{x}" y="{y}"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>
<p:txBody><a:bodyPr wrap="square" rtlCol="0"><a:normAutofit/></a:bodyPr><a:lstStyle/>
<a:p><a:r><a:rPr lang="fr-FR" sz="{sz}" b="{b}" dirty="0"/><a:t>{text}</a:t></a:r></a:p>
</p:txBody></p:sp>"#,
        name_id = if bold { 2 } else { 3 },
    )
}

fn slide_rels(idx: usize) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image{idx}.png"/>
</Relationships>"#
    )
}

/// Ajuste `(img)` px dans `(box)` EMU en conservant le rapport d'aspect.
fn fit_emu(img_w: i64, img_h: i64, box_w: i64, box_h: i64) -> (i64, i64) {
    // Compare les rapports via produits croisés pour rester en entiers.
    if img_w * box_h <= box_w * img_h {
        // Limité par la hauteur.
        (img_w * box_h / img_h, box_h)
    } else {
        // Limité par la largeur.
        (box_w, img_h * box_w / img_w)
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ----- Parties statiques (thème, masque, mise en page) -----------------------

const SLIDE_MASTER_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="../theme/theme1.xml"/>
</Relationships>"#;

const SLIDE_LAYOUT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="../slideMasters/slideMaster1.xml"/>
</Relationships>"#;

const SLIDE_MASTER: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld><p:bg><p:bgRef idx="1001"><a:schemeClr val="bg1"/></p:bgRef></p:bg>
<p:spTree>
<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
<p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
</p:spTree></p:cSld>
<p:clrMap bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/>
<p:sldLayoutIdLst><p:sldLayoutId id="2147483649" r:id="rId1"/></p:sldLayoutIdLst>
<p:txStyles>
<p:titleStyle><a:lvl1pPr><a:defRPr sz="4400"/></a:lvl1pPr></p:titleStyle>
<p:bodyStyle><a:lvl1pPr><a:defRPr sz="1800"/></a:lvl1pPr></p:bodyStyle>
<p:otherStyle/>
</p:txStyles>
</p:sldMaster>"#;

const SLIDE_LAYOUT: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" type="blank" preserve="1">
<p:cSld name="Vide"><p:spTree>
<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
<p:grpSpPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="0" cy="0"/><a:chOff x="0" y="0"/><a:chExt cx="0" cy="0"/></a:xfrm></p:grpSpPr>
</p:spTree></p:cSld>
<p:clrMapOvr><a:overrideClrMapping bg1="lt1" tx1="dk1" bg2="lt2" tx2="dk2" accent1="accent1" accent2="accent2" accent3="accent3" accent4="accent4" accent5="accent5" accent6="accent6" hlink="hlink" folHlink="folHlink"/></p:clrMapOvr>
</p:sldLayout>"#;

const THEME: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="GEVSTUDIO">
<a:themeElements>
<a:clrScheme name="GEVSTUDIO">
<a:dk1><a:sysClr val="windowText" lastClr="000000"/></a:dk1>
<a:lt1><a:sysClr val="window" lastClr="FFFFFF"/></a:lt1>
<a:dk2><a:srgbClr val="23262E"/></a:dk2>
<a:lt2><a:srgbClr val="F6F4FB"/></a:lt2>
<a:accent1><a:srgbClr val="1D4ED8"/></a:accent1>
<a:accent2><a:srgbClr val="22A34F"/></a:accent2>
<a:accent3><a:srgbClr val="F59117"/></a:accent3>
<a:accent4><a:srgbClr val="C6302C"/></a:accent4>
<a:accent5><a:srgbClr val="1A73E8"/></a:accent5>
<a:accent6><a:srgbClr val="7E96F7"/></a:accent6>
<a:hlink><a:srgbClr val="1D4ED8"/></a:hlink>
<a:folHlink><a:srgbClr val="6E727D"/></a:folHlink>
</a:clrScheme>
<a:fontScheme name="GEVSTUDIO">
<a:majorFont><a:latin typeface="DejaVu Sans"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont>
<a:minorFont><a:latin typeface="DejaVu Sans"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont>
</a:fontScheme>
<a:fmtScheme name="GEVSTUDIO">
<a:fillStyleLst>
<a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
<a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
<a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
</a:fillStyleLst>
<a:lnStyleLst>
<a:ln w="6350"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>
<a:ln w="12700"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>
<a:ln w="19050"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln>
</a:lnStyleLst>
<a:effectStyleLst>
<a:effectStyle><a:effectLst/></a:effectStyle>
<a:effectStyle><a:effectLst/></a:effectStyle>
<a:effectStyle><a:effectLst/></a:effectStyle>
</a:effectStyleLst>
<a:bgFillStyleLst>
<a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
<a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
<a:solidFill><a:schemeClr val="phClr"/></a:solidFill>
</a:bgFillStyleLst>
</a:fmtScheme>
</a:themeElements>
</a:theme>"#;

#[cfg(test)]
mod tests {
    use super::*;
    use gev_core::document::Document;
    use gev_core::model::{Annotation, Shape};

    #[test]
    fn build_produces_valid_pptx_package() {
        let mut image = image::RgbaImage::new(64, 40);
        for (x, _y, px) in image.enumerate_pixels_mut() {
            *px = image::Rgba([(x * 4) as u8, 90, 180, 255]);
        }
        let step = Step {
            image,
            annotations: vec![Annotation {
                shape: Shape::Rect {
                    rect: emath::Rect::from_min_size(emath::pos2(5.0, 5.0), emath::vec2(20.0, 15.0)),
                },
                color: ecolor::Color32::RED,
                stroke_width: 2.0,
            }],
            title: "Étape <1> & « test »".into(),
            caption: "Légende".into(),
            narration: String::new(),
            narration_audio: None,
            duration_override_ms: None,
            transition: Transition::Cut,
        };
        let doc = Document {
            title: "Guide".into(),
            steps: vec![step],
            voice_id: String::new(),
        };
        let bytes = build(&doc).expect("génération PPTX");
        // En-tête ZIP local (PK\x03\x04).
        assert_eq!(&bytes[..4], b"PK\x03\x04");

        // Le paquet doit contenir les parties clés.
        let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).expect("archive lisible");
        let names: Vec<String> = (0..zip.len())
            .map(|i| zip.by_index(i).unwrap().name().to_string())
            .collect();
        for part in [
            "[Content_Types].xml",
            "ppt/presentation.xml",
            "ppt/slides/slide1.xml",
            "ppt/media/image1.png",
        ] {
            assert!(names.iter().any(|n| n == part), "partie manquante : {part}");
        }
    }
}
