//! Police d'interface Inter (Direction B), partagée entre GEVSHOT et
//! GEVSTUDIO. Les applications appellent `install_ui` telle quelle, ou
//! composent `add_inter` avec leurs propres familles avant `set_fonts`.

use eframe::egui;

/// Ajoute Inter aux définitions de polices : première police de la famille
/// proportionnelle + familles nommées `inter-medium` / `inter-semibold`.
pub fn add_inter(fonts: &mut egui::FontDefinitions) {
    for (name, bytes) in [
        ("inter", &include_bytes!("../assets/fonts/Inter-Regular.ttf")[..]),
        (
            "inter-medium",
            &include_bytes!("../assets/fonts/Inter-Medium.ttf")[..],
        ),
        (
            "inter-semibold",
            &include_bytes!("../assets/fonts/Inter-SemiBold.ttf")[..],
        ),
    ] {
        fonts.font_data.insert(
            name.to_string(),
            std::sync::Arc::new(egui::FontData::from_static(bytes)),
        );
    }
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "inter".to_string());
    fonts.families.insert(
        egui::FontFamily::Name("inter-semibold".into()),
        vec!["inter-semibold".to_string()],
    );
    fonts.families.insert(
        egui::FontFamily::Name("inter-medium".into()),
        vec!["inter-medium".to_string()],
    );
}

/// Installe Inter seule (application sans familles supplémentaires).
pub fn install_ui(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    add_inter(&mut fonts);
    ctx.set_fonts(fonts);
}
