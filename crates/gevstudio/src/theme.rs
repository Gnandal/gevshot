//! Thème « Direction B — Minimaliste & sobre » : neutres chauds + un seul
//! accent bleu réservé à la sélection et au focus (jamais décoratif). Le noir
//! encre porte les actions. Typo Inter.

use eframe::egui::{self, Color32, FontFamily, RichText};

// ----- Palette ---------------------------------------------------------------

pub const WHITE: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF); // surfaces
pub const SUBTLE: Color32 = Color32::from_rgb(0xFA, 0xFA, 0xF9); // fond doux
pub const SURFACE_2: Color32 = Color32::from_rgb(0xF5, 0xF5, 0xF4); // survol
pub const HAIRLINE: Color32 = Color32::from_rgb(0xE7, 0xE5, 0xE4); // bordures
pub const MUTED: Color32 = Color32::from_rgb(0x79, 0x71, 0x6C); // texte secondaire
pub const INK: Color32 = Color32::from_rgb(0x1C, 0x19, 0x17); // texte + actions
pub const ACCENT: Color32 = Color32::from_rgb(0x25, 0x63, 0xEB); // sélection / focus
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0xDB, 0xEA, 0xFE);
pub const DANGER: Color32 = Color32::from_rgb(0xB4, 0x23, 0x18);
pub const SUCCESS: Color32 = Color32::from_rgb(0x1E, 0x9A, 0x4E);

/// Fond de la zone d'aperçu.
pub const CANVAS: Color32 = SUBTLE;

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

        let radius = egui::CornerRadius::same(10);
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

        // Boutons/champs « secondaires » : surface claire, bordure fine.
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

        // Focus : liseré accent (seul emploi décoratif toléré du bleu).
        v.widgets.hovered.expansion = 0.0;
        v.widgets.active.expansion = 0.0;

        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(14.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(0);
    });
}

// ----- Boutons ---------------------------------------------------------------

/// Bouton primaire : encre pleine, texte blanc (action forte).
pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(medium(text).color(WHITE))
            .fill(INK)
            .corner_radius(egui::CornerRadius::same(10)),
    )
}

/// Bouton destructif : texte rouge, fond clair.
pub fn danger_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.add(egui::Button::new(RichText::new(text).color(DANGER)))
}
