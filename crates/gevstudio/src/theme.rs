//! Thème de l'interface : même palette lavande claire que GEVSHOT.

use eframe::egui::{self, Color32};

pub const BG_PANEL: Color32 = Color32::from_rgb(0xF6, 0xF4, 0xFB);
pub const BG_CANVAS: Color32 = Color32::from_rgb(0xEC, 0xEA, 0xF4);
pub const BG_HOVER: Color32 = Color32::from_rgb(0xEA, 0xE7, 0xF3);
pub const ACCENT: Color32 = Color32::from_rgb(0x1D, 0x4E, 0xD8);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0xD9, 0xE6, 0xFA);
pub const TEXT: Color32 = Color32::from_rgb(0x23, 0x26, 0x2E);
pub const BORDER: Color32 = Color32::from_rgb(0xDD, 0xDA, 0xE7);

pub fn apply(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Light);
    ctx.all_styles_mut(|style| {
        let v = &mut style.visuals;
        *v = egui::Visuals::light();
        v.panel_fill = BG_PANEL;
        v.window_fill = BG_PANEL;
        v.extreme_bg_color = Color32::WHITE;
        v.faint_bg_color = BG_HOVER;
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
        v.widgets.noninteractive.bg_fill = BG_PANEL;
        v.widgets.noninteractive.fg_stroke.color = TEXT;
        v.widgets.noninteractive.bg_stroke.color = BORDER;
        v.widgets.inactive.bg_fill = Color32::from_rgb(0xEE, 0xEC, 0xF6);
        v.widgets.inactive.weak_bg_fill = Color32::from_rgb(0xEE, 0xEC, 0xF6);
        v.widgets.inactive.fg_stroke.color = TEXT;
        v.widgets.hovered.bg_fill = BG_HOVER;
        v.widgets.hovered.weak_bg_fill = BG_HOVER;
        v.widgets.hovered.fg_stroke.color = TEXT;
        v.widgets.active.bg_fill = ACCENT_SOFT;
        v.widgets.active.weak_bg_fill = ACCENT_SOFT;
        v.widgets.active.fg_stroke.color = TEXT;

        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
    });
}
