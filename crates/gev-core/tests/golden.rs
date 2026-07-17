//! Tests golden : le `.gev` de référence (généré par GEVSHOT v1.0.2 avant
//! l'extraction de gev-core) doit se relire à l'identique et s'aplatir au
//! pixel près. `BLESS=1 cargo test -p gev-core` régénère le PNG attendu.

use std::path::{Path, PathBuf};

use gev_core::fonts::FontSet;
use gev_core::{project, render};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn golden_gev_round_trip() {
    let (image, annotations) =
        project::load(&fixture("golden.gev")).expect("lecture du .gev de référence");
    assert_eq!((image.width(), image.height()), (640, 400));
    assert_eq!(annotations.len(), 8);

    // Aller-retour : la sérialisation doit être stable.
    let tmp = std::env::temp_dir().join("gev_core_golden_roundtrip.gev");
    project::save(&tmp, &image, &annotations).expect("écriture");
    let (image2, annotations2) = project::load(&tmp).expect("relecture");
    assert_eq!(image.as_raw(), image2.as_raw());
    assert_eq!(annotations, annotations2);
}

#[test]
fn golden_flatten_pixels() {
    let (image, annotations) = project::load(&fixture("golden.gev")).expect("lecture");
    // Jeu de polices vide : rendu 100 % vectoriel, déterministe sur toute
    // machine (le texte dépend des fontes système, vérifié hors CI).
    let flat = render::flatten(&image, &annotations, &FontSet::empty());
    let expected_path = fixture("golden_flat_shapes.png");
    if std::env::var_os("BLESS").is_some() {
        flat.save(&expected_path).expect("écriture du PNG attendu");
        return;
    }
    let expected = image::open(&expected_path)
        .expect("PNG attendu manquant — lancer avec BLESS=1")
        .to_rgba8();
    assert_eq!(
        flat.as_raw(),
        expected.as_raw(),
        "l'aplatissement des formes a changé"
    );
}

#[test]
fn rejects_newer_version() {
    let tmp = std::env::temp_dir().join("gev_core_golden_v2.gev");
    std::fs::write(&tmp, r#"{"version":2,"image_png_base64":"","annotations":[]}"#).unwrap();
    assert!(project::load(&tmp).is_err());
}
