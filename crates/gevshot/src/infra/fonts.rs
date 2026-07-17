//! Enregistrement des polices système dans egui (éditeur) et jeu de
//! polices d'export partagé. La résolution des fichiers vit dans
//! `gev_core::fonts`.

use std::sync::OnceLock;

use eframe::egui;
use gev_core::fonts::{FontSet, family_name, system_font_bytes};
use gev_core::model::FontChoice;

/// Enregistre les familles système dans egui (éditeur).
/// Les familles introuvables retombent sur la police proportionnelle par défaut.
pub fn install(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let default_family = fonts
        .families
        .get(&egui::FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    for font in FontChoice::ALL {
        for bold in [false, true] {
            let name = family_name(font, bold);
            let family = egui::FontFamily::Name(name.into());
            match system_font_bytes(font, bold) {
                Some(bytes) => {
                    fonts.font_data.insert(
                        name.to_string(),
                        std::sync::Arc::new(egui::FontData::from_owned(bytes)),
                    );
                    fonts.families.insert(family, vec![name.to_string()]);
                }
                None => {
                    fonts.families.insert(family, default_family.clone());
                }
            }
        }
    }
    ctx.set_fonts(fonts);
}

/// Jeu de polices d'export (système), initialisé une fois.
pub fn export_fonts() -> &'static FontSet {
    static SET: OnceLock<FontSet> = OnceLock::new();
    SET.get_or_init(FontSet::system)
}
