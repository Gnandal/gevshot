//! Widgets réutilisables (adapté de GEVSHOT).
//! Stubs v1 pour icon_button, pill_button, swatch.
//! Implémentations complètes avec peintres à venir.

use eframe::egui::{self, Ui};

/// Bouton à icône (34×30, style GEVSHOT).
/// v1 : stub utilisant un simple bouton egui
pub fn icon_button(ui: &mut Ui, _icon_fn: impl Fn(&mut egui::Painter, egui::Rect)) -> egui::Response {
    // v1 : placeholder button
    // v2 : custom paint via icon_fn
    ui.button("⊕")
}

/// Bouton important (style primaire).
pub fn pill_button(ui: &mut Ui, label: &str, enabled: bool) -> egui::Response {
    ui.add_enabled(enabled, egui::Button::new(label))
}

/// Sélecteur de couleur (swatch).
pub fn swatch(ui: &mut Ui, color: &mut ecolor::Color32) -> egui::Response {
    let size = egui::vec2(20.0, 20.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    ui.painter().rect_filled(rect, 2.0, *color);
    resp
}
