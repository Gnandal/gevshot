use eframe::egui::{self, Color32};

// Thème « Direction B — Minimaliste & sobre » : neutres chauds + un seul
// accent bleu réservé à la sélection et au focus. Le noir encre porte les
// actions. Partagé avec GEVSTUDIO.
pub const BG_TOOLBAR: Color32 = Color32::from_rgb(0xFF, 0xFF, 0xFF); // surfaces
pub const BG_CANVAS: Color32 = Color32::from_rgb(0xFA, 0xFA, 0xF9); // zone d'édition
pub const BG_HOVER: Color32 = Color32::from_rgb(0xF5, 0xF5, 0xF4);
pub const ACCENT: Color32 = Color32::from_rgb(0x25, 0x63, 0xEB); // sélection / focus
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0xDB, 0xEA, 0xFE);
pub const TEXT: Color32 = Color32::from_rgb(0x1C, 0x19, 0x17);
pub const TEXT_WEAK: Color32 = Color32::from_rgb(0x79, 0x71, 0x6C);
pub const BORDER: Color32 = Color32::from_rgb(0xE7, 0xE5, 0xE4);
pub const SUCCESS: Color32 = Color32::from_rgb(0x1E, 0x9A, 0x4E);
pub const DANGER: Color32 = Color32::from_rgb(0xC0, 0x39, 0x2B);

// Pilule d'aide affichée pendant la sélection de région (overlay sombre sur la
// capture plein écran — inchangé, indépendant du thème clair de l'app).
pub const PILL_BG: Color32 = Color32::from_rgb(0x1B, 0x20, 0x2B);
pub const PILL_ICON: Color32 = Color32::from_rgb(0x7E, 0x96, 0xF7);
pub const PILL_KEY_TEXT: Color32 = Color32::from_rgb(0xC8, 0xCC, 0xD4);
pub const PILL_KEY_BORDER: Color32 = Color32::from_rgb(0x4A, 0x51, 0x60);

/// Couleurs d'annotation proposées dans la barre d'outils (contenu, pas UI).
pub const PALETTE: [Color32; 5] = [
    Color32::from_rgb(0xC6, 0x30, 0x2C), // rouge
    Color32::from_rgb(0x1A, 0x73, 0xE8), // bleu
    Color32::from_rgb(0xF5, 0x91, 0x17), // orange
    Color32::from_rgb(0x22, 0xA3, 0x4F), // vert
    Color32::from_rgb(0x17, 0x18, 0x1C), // noir
];

/// Épaisseurs de trait proposées dans la barre d'outils.
pub const STROKES: [f32; 4] = [2.0, 3.5, 5.0, 8.0];

pub fn apply(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Light);
    ctx.all_styles_mut(|style| {
        let v = &mut style.visuals;
        *v = egui::Visuals::light();
        v.panel_fill = BG_TOOLBAR;
        v.window_fill = BG_TOOLBAR;
        v.window_stroke = egui::Stroke::new(1.0, BORDER);
        v.extreme_bg_color = Color32::WHITE;
        v.faint_bg_color = BG_CANVAS;
        v.override_text_color = Some(TEXT);
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
        v.widgets.noninteractive.bg_fill = BG_TOOLBAR;
        v.widgets.noninteractive.fg_stroke.color = TEXT;
        v.widgets.noninteractive.bg_stroke.color = BORDER;

        v.widgets.inactive.bg_fill = Color32::WHITE;
        v.widgets.inactive.weak_bg_fill = Color32::WHITE;
        v.widgets.inactive.fg_stroke.color = TEXT;
        v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, BORDER);

        v.widgets.hovered.bg_fill = BG_HOVER;
        v.widgets.hovered.weak_bg_fill = BG_HOVER;
        v.widgets.hovered.fg_stroke.color = TEXT;
        v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, BORDER);

        v.widgets.active.bg_fill = BG_HOVER;
        v.widgets.active.weak_bg_fill = BG_HOVER;
        v.widgets.active.fg_stroke.color = TEXT;
        v.widgets.active.bg_stroke = egui::Stroke::new(1.0, ACCENT);

        // Fenêtres (modales) : coins arrondis, bordure fine, sans ombre.
        v.window_corner_radius = egui::CornerRadius::same(12);
        v.window_shadow = egui::epaint::Shadow::NONE;
        v.popup_shadow = egui::epaint::Shadow::NONE;

        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
    });
}
