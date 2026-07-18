//! Thème « esprit macOS » : chrome gris clair (barre latérale/outils) sur
//! contenu blanc, bleu système pour la sélection et les actions par défaut,
//! coins arrondis généreux, ombres douces, espacements aérés. Typo Inter
//! (substitut de SF).

use eframe::egui::{self, Color32, FontFamily, RichText};

// ----- Palette (macOS light) -------------------------------------------------

pub const WHITE: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF); // contenu
pub const SUBTLE: Color32 = Color32::from_rgb(0xF5, 0xF5, 0xF7); // chrome gris
pub const SURFACE_2: Color32 = Color32::from_rgb(0xEA, 0xEA, 0xEF); // survol
pub const HAIRLINE: Color32 = Color32::from_rgb(0xE3, 0xE3, 0xE6); // séparateurs
pub const MUTED: Color32 = Color32::from_rgb(0x86, 0x86, 0x8B); // libellé secondaire
pub const INK: Color32 = Color32::from_rgb(0x1D, 0x1D, 0x1F); // libellé principal
pub const ACCENT: Color32 = Color32::from_rgb(0x00, 0x7A, 0xFF); // bleu système
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0xE4, 0xF0, 0xFF); // surbrillance
pub const DANGER: Color32 = Color32::from_rgb(0xFF, 0x3B, 0x30); // rouge système
pub const SUCCESS: Color32 = Color32::from_rgb(0x34, 0xC7, 0x59); // vert système

/// Fond de la zone d'aperçu.
pub const CANVAS: Color32 = SUBTLE;

/// Ombre douce façon carte macOS.
pub fn card_shadow() -> egui::epaint::Shadow {
    egui::epaint::Shadow {
        offset: [0, 2],
        blur: 12,
        spread: 0,
        color: Color32::from_black_alpha(24),
    }
}

const SEMIBOLD: &str = "inter-semibold";
const MEDIUM: &str = "inter-medium";

// ----- Typo ------------------------------------------------------------------

/// Installe Inter comme police d'interface (Regular par défaut, familles
/// nommées « semibold » et « medium »).
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let mut add = |name: &str, bytes: &'static [u8]| {
        fonts.font_data.insert(
            name.to_string(),
            std::sync::Arc::new(egui::FontData::from_static(bytes)),
        );
    };
    add("inter", include_bytes!("../assets/fonts/Inter-Regular.ttf"));
    add(MEDIUM, include_bytes!("../assets/fonts/Inter-Medium.ttf"));
    add(SEMIBOLD, include_bytes!("../assets/fonts/Inter-SemiBold.ttf"));

    // Inter devient la police proportionnelle par défaut (avant les polices de
    // repli, pour conserver un rendu cohérent sur les caractères manquants).
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "inter".to_string());
    fonts
        .families
        .insert(FontFamily::Name(SEMIBOLD.into()), vec![SEMIBOLD.to_string()]);
    fonts
        .families
        .insert(FontFamily::Name(MEDIUM.into()), vec![MEDIUM.to_string()]);

    ctx.set_fonts(fonts);
}

/// Titre / libellé fort (Inter SemiBold).
pub fn semibold(text: impl Into<String>) -> RichText {
    RichText::new(text).family(FontFamily::Name(SEMIBOLD.into()))
}

/// Libellé moyen (Inter Medium).
pub fn medium(text: impl Into<String>) -> RichText {
    RichText::new(text).family(FontFamily::Name(MEDIUM.into()))
}

/// Méta en petites capitales grises.
pub fn meta(text: &str) -> RichText {
    medium(text.to_uppercase()).size(11.0).color(MUTED)
}

// ----- Visuals ---------------------------------------------------------------

pub fn apply(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Light);
    ctx.all_styles_mut(|style| {
        let v = &mut style.visuals;
        *v = egui::Visuals::light();
        v.panel_fill = WHITE;
        v.window_fill = WHITE;
        v.window_stroke = egui::Stroke::new(1.0, HAIRLINE);
        v.extreme_bg_color = WHITE;
        v.faint_bg_color = SUBTLE;
        v.override_text_color = Some(INK);
        v.selection.bg_fill = ACCENT_SOFT;
        v.selection.stroke = egui::Stroke::new(1.0, ACCENT);
        v.hyperlink_color = ACCENT;

        let radius = egui::CornerRadius::same(8);
        for w in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            w.corner_radius = radius;
        }
        v.widgets.noninteractive.bg_fill = WHITE;
        v.widgets.noninteractive.fg_stroke.color = INK;
        v.widgets.noninteractive.bg_stroke.color = HAIRLINE;

        // Boutons/champs « secondaires » : surface claire, bordure discrète.
        v.widgets.inactive.bg_fill = WHITE;
        v.widgets.inactive.weak_bg_fill = WHITE;
        v.widgets.inactive.fg_stroke.color = INK;
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, HAIRLINE);

        v.widgets.hovered.bg_fill = SURFACE_2;
        v.widgets.hovered.weak_bg_fill = SURFACE_2;
        v.widgets.hovered.fg_stroke.color = INK;
        v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, HAIRLINE);

        v.widgets.active.bg_fill = SURFACE_2;
        v.widgets.active.weak_bg_fill = SURFACE_2;
        v.widgets.active.fg_stroke.color = INK;
        v.widgets.active.bg_stroke = egui::Stroke::new(1.0, ACCENT);
        v.widgets.hovered.expansion = 0.0;
        v.widgets.active.expansion = 0.0;

        // Fenêtres (modales) : coins arrondis + ombre douce macOS.
        v.window_corner_radius = egui::CornerRadius::same(14);
        v.window_shadow = card_shadow();
        v.popup_shadow = card_shadow();

        // Espacements aérés.
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(16.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(0);
    });
}

// ----- Boutons ---------------------------------------------------------------

/// Bouton primaire (bouton par défaut macOS) : bleu système, texte blanc.
pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(medium(text).color(WHITE))
            .fill(ACCENT)
            .corner_radius(egui::CornerRadius::same(8)),
    )
}

/// Bouton destructif : texte rouge, fond clair.
pub fn danger_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(egui::Button::new(RichText::new(text).color(DANGER)))
}
