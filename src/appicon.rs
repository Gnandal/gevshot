// Icône de l'application, dessinée par code (aucun fichier d'asset).
// Ce module est volontairement autonome (tiny-skia uniquement) : il est
// aussi inclus par build.rs pour générer le .ico embarqué dans l'exe —
// d'où des commentaires « // » (les « //! » seraient invalides une fois
// le fichier inclus au milieu de build.rs).

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

fn circle_path(cx: f32, cy: f32, r: f32) -> tiny_skia::Path {
    let mut pb = PathBuilder::new();
    pb.push_circle(cx, cy, r);
    pb.finish().expect("cercle invalide")
}

fn fill(pm: &mut Pixmap, path: &tiny_skia::Path, rgb: (u8, u8, u8)) {
    let mut paint = Paint::default();
    paint.anti_alias = true;
    paint.set_color_rgba8(rgb.0, rgb.1, rgb.2, 255);
    pm.fill_path(path, &paint, FillRule::Winding, Transform::identity(), None);
}

/// Dessine l'icône GEVSHOT : appareil photo blanc sur fond bleu arrondi,
/// objectif bleu, témoin rouge.
pub fn draw_app_icon(size: u32) -> Pixmap {
    let mut pm = Pixmap::new(size, size).expect("pixmap");
    let s = size as f32;
    // Fond : carré arrondi bleu accent.
    fill(
        &mut pm,
        &rounded_rect_path(0.02 * s, 0.02 * s, 0.96 * s, 0.96 * s, 0.21 * s),
        (0x1D, 0x4E, 0xD8),
    );
    // Boîtier de l'appareil.
    fill(
        &mut pm,
        &rounded_rect_path(0.16 * s, 0.34 * s, 0.68 * s, 0.42 * s, 0.07 * s),
        (0xFF, 0xFF, 0xFF),
    );
    // Bosse du viseur.
    fill(
        &mut pm,
        &rounded_rect_path(0.36 * s, 0.24 * s, 0.28 * s, 0.14 * s, 0.05 * s),
        (0xFF, 0xFF, 0xFF),
    );
    // Objectif : disque bleu, pupille bleu clair.
    fill(
        &mut pm,
        &circle_path(0.5 * s, 0.55 * s, 0.155 * s),
        (0x1D, 0x4E, 0xD8),
    );
    fill(
        &mut pm,
        &circle_path(0.5 * s, 0.55 * s, 0.085 * s),
        (0xD9, 0xE6, 0xFA),
    );
    // Témoin rouge.
    fill(
        &mut pm,
        &circle_path(0.735 * s, 0.415 * s, 0.038 * s),
        (0xE0, 0x28, 0x28),
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
