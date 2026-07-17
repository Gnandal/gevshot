#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! GEVSHOT — capture d'écran annotée pour la documentation logicielle.
//!
//! Architecture en couches :
//! - `domain` : modèle d'annotations et état de l'éditeur (métier pur) ;
//! - `infra`  : accès système et réseau (capture, fichiers, config, IA…) ;
//! - `ui`     : thème, widgets et écrans réutilisables ;
//! - `app`    : orchestration — état global, flux de capture, barres, panneaux.
//!
//! Le modèle des annotations, le format `.gev` et l'aplatissement sur
//! pixels vivent dans la crate partagée `gev-core`.

mod app;
mod appicon;
mod domain;
mod infra;
mod ui;

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
            ui::theme::apply(&cc.egui_ctx);
            infra::fonts::install(&cc.egui_ctx);
            Ok(Box::new(app::GevshotApp::new()))
        }),
    )
}
