// Icône de l'application, dessinée par code (aucun fichier d'asset), sur le
// modèle de gevshot/src/appicon.rs : autonome (tiny-skia uniquement) pour
// pouvoir être incluse par un build.rs plus tard.

use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Transform};

fn rounded_rect_path(x: f32, y: f32, w: f32, h: f32, r: f32) -> tiny_skia::Path {
    let (x2, y2) = (x + w, y + h);
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x2 - r, y);
    pb.quad_to(x2, y, x2, y + r);
    pb.line_to(x2, y2 - r);
    pb.quad_to(x2, y2, x2 - r, y2);
    pb.line_to(x + r, y2);
    pb.quad_to(x, y2, x, y2 - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
    pb.finish().expect("rectangle arrondi invalide")
}

fn triangle_path(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> tiny_skia::Path {
    let mut pb = PathBuilder::new();
    pb.move_to(a.0, a.1);
    pb.line_to(b.0, b.1);
    pb.line_to(c.0, c.1);
    pb.close();
    pb.finish().expect("triangle invalide")
}

fn fill(pm: &mut Pixmap, path: &tiny_skia::Path, rgb: (u8, u8, u8)) {
    let mut paint = Paint {
        anti_alias: true,
        ..Paint::default()
    };
    paint.set_color_rgba8(rgb.0, rgb.1, rgb.2, 255);
    pm.fill_path(path, &paint, FillRule::Winding, Transform::identity(), None);
}

/// Dessine l'icône GEVSTUDIO : pellicule blanche sur fond bleu arrondi,
/// triangle de lecture bleu au centre.
pub fn draw_app_icon(size: u32) -> Pixmap {
    let mut pm = Pixmap::new(size, size).expect("pixmap");
    let s = size as f32;
    // Fond : carré arrondi bleu accent (même bleu que GEVSHOT).
    fill(
        &mut pm,
        &rounded_rect_path(0.02 * s, 0.02 * s, 0.96 * s, 0.96 * s, 0.21 * s),
        (0x1D, 0x4E, 0xD8),
    );
    // Cadre « pellicule » blanc.
    fill(
        &mut pm,
        &rounded_rect_path(0.16 * s, 0.26 * s, 0.68 * s, 0.48 * s, 0.07 * s),
        (0xFF, 0xFF, 0xFF),
    );
    // Perforations latérales (bleu du fond).
    for i in 0..3 {
        let y = (0.31 + 0.15 * i as f32) * s;
        fill(
            &mut pm,
            &rounded_rect_path(0.19 * s, y, 0.055 * s, 0.075 * s, 0.015 * s),
            (0x1D, 0x4E, 0xD8),
        );
        fill(
            &mut pm,
            &rounded_rect_path(0.755 * s, y, 0.055 * s, 0.075 * s, 0.015 * s),
            (0x1D, 0x4E, 0xD8),
        );
    }
    // Triangle de lecture.
    fill(
        &mut pm,
        &triangle_path(
            (0.42 * s, 0.36 * s),
            (0.42 * s, 0.64 * s),
            (0.66 * s, 0.50 * s),
        ),
        (0x1D, 0x4E, 0xD8),
    );
    pm
}

/// Pixels RGBA (alpha non prémultiplié) de l'icône à la taille demandée.
pub fn icon_rgba(size: u32) -> Vec<u8> {
    let pm = draw_app_icon(size);
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    for px in pm.pixels() {
        let d = px.demultiply();
        out.extend_from_slice(&[d.red(), d.green(), d.blue(), d.alpha()]);
    }
    out
}
