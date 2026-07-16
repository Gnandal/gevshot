#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ai;
mod annotation;
mod app;
mod appicon;
mod capture;
mod config;
mod editor;
mod export;
mod fonts;
mod project;
mod settings_ui;
mod theme;
mod tray;
mod widgets;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([640.0, 480.0])
            .with_title("GEVSHOT")
            .with_icon(egui::IconData {
                rgba: appicon::icon_rgba(64),
                width: 64,
                height: 64,
            })
            // Démarrage silencieux : l'appli vit dans la barre système,
            // la fenêtre n'apparaît qu'après une capture.
            .with_visible(false),
        ..Default::default()
    };
    eframe::run_native(
        "GEVSHOT",
        options,
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx);
            fonts::install(&cc.egui_ctx);
            Ok(Box::new(app::GevshotApp::new()))
        }),
    )
}
