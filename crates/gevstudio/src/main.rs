#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! GEVSTUDIO — assemble des captures annotées en guides pas-à-pas
//! (Document A4, Slides 16:9, Vidéo), exportés en PDF/PPTX/MP4.
//!
//! Architecture en couches, comme GEVSHOT :
//! - `domain` : durées, séquencement (métier pur) ;
//! - `infra`  : fichiers, config, exports, TTS ;
//! - `ui`     : icônes, toasts, widgets propres à GEVSTUDIO ;
//! - `app`    : orchestration — état global, barres, panneaux, scènes.
//!
//! Le modèle du document (`.gevd`) vit dans `gev-core`, les briques
//! d'interface partagées (thème, éditeur d'annotations) dans `gev-ui`.

use std::path::PathBuf;

use eframe::egui;

mod app;
mod appicon;
mod domain;
mod infra;
mod license;
mod ui;

fn main() -> eframe::Result {
    // Un chemin `.gevd` en argument est ouvert au démarrage.
    let open_path = std::env::args().nth(1).map(PathBuf::from);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([980.0, 600.0])
            .with_title("GevShot Studio")
            .with_icon(egui::IconData {
                rgba: appicon::icon_rgba(64),
                width: 64,
                height: 64,
            }),
        ..Default::default()
    };
    eframe::run_native(
        "GEVSTUDIO",
        options,
        Box::new(move |cc| {
            gev_ui::theme::apply(&cc.egui_ctx);
            gev_ui::fonts::install_ui(&cc.egui_ctx);
            Ok(Box::new(app::StudioApp::new(open_path)))
        }),
    )
}
