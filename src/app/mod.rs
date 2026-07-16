//! Couche application : l'état global (`GevshotApp`), le cycle de vie
//! eframe, et l'orchestration des flux. Chaque sous-module couvre un
//! aspect : capture, sélection de région, barres, panneaux, IA, fichiers.

mod ai_flow;
mod capture_flow;
mod files;
mod legend;
mod region_select;
mod settings_modal;
mod status_bar;
mod toolbar;
mod window_picker;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui;
use global_hotkey::{
    GlobalHotKeyManager,
    hotkey::{Code, HotKey, Modifiers},
};

use crate::domain::editor::Editor;
use crate::infra::config::Settings;
use crate::infra::tray::Tray;
use crate::ui::settings::SettingsDraft;
use crate::ui::theme;

/// Mode de la prochaine capture.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CaptureMode {
    /// Région tracée à la souris sur l'écran gelé.
    Region,
    /// Écran entier, sans étape de sélection.
    FullScreen,
    /// Fenêtre au premier plan.
    Window,
    /// Fenêtre précise, choisie dans le sélecteur.
    WindowById(u32),
}

/// Étape courante de l'application.
enum Stage {
    Empty,
    /// Plein écran : l'utilisateur choisit la zone sur la capture gelée.
    /// `prev` restaure la session d'édition si la sélection est annulée.
    Selecting {
        full: Capture,
        prev: Option<Capture>,
    },
    Editing {
        capture: Capture,
    },
}

/// Une image capturée et sa texture GPU.
pub struct Capture {
    pub image: image::RgbaImage,
    pub texture: egui::TextureHandle,
}

impl Capture {
    fn from_image(ctx: &egui::Context, image: image::RgbaImage) -> Self {
        let color_image = egui::ColorImage::from_rgba_unmultiplied(
            [image.width() as usize, image.height() as usize],
            image.as_raw(),
        );
        let texture = ctx.load_texture("capture", color_image, egui::TextureOptions::LINEAR);
        Self { image, texture }
    }

    fn size(&self) -> egui::Vec2 {
        egui::vec2(self.image.width() as f32, self.image.height() as f32)
    }
}

pub struct GevshotApp {
    stage: Stage,
    editor: Editor,
    pending_capture: Option<Instant>,
    pending_mode: CaptureMode,
    /// Délai effectif avant la prise de vue (masquage + éventuel minuteur).
    pending_delay_ms: u64,
    error: Option<String>,
    status: Option<String>,
    /// Rectangle de sélection en cours (coordonnées écran).
    selection: Option<(egui::Pos2, egui::Pos2)>,
    tray: Option<Tray>,
    _hotkey_manager: Option<GlobalHotKeyManager>,
    hotkey_id: Option<u32>,
    /// La prochaine capture conserve les annotations (recapture du fond).
    keep_annotations: bool,
    quitting: bool,
    first_frame: bool,
    /// La fenêtre a été dimensionnée/centrée sur l'écran.
    positioned: bool,
    settings: Settings,
    settings_open: bool,
    settings_draft: Option<SettingsDraft>,
    /// Panneau latéral de légende (textes des pastilles).
    legend_open: bool,
    /// Réception du résultat d'un appel IA en cours.
    ai_rx: Option<std::sync::mpsc::Receiver<crate::infra::ai::AiResult>>,
    /// Message affiché pendant un appel IA.
    ai_busy: Option<String>,
    /// Instruction saisie pour « Annoter par IA ».
    ai_prompt: String,
    /// Sélecteur de fenêtre à capturer (liste ouverte).
    window_picker: Option<Vec<crate::infra::capture::WindowInfo>>,
    /// Nom du document courant, affiché dans le titre de la fenêtre.
    doc_name: String,
}

impl GevshotApp {
    pub fn new() -> Self {
        let mut error = None;
        let tray = match Tray::new() {
            Ok(tray) => Some(tray),
            Err(e) => {
                error = Some(e);
                None
            }
        };
        let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyS);
        let (manager, hotkey_id) =
            match GlobalHotKeyManager::new().and_then(|m| m.register(hotkey).map(|()| m)) {
                Ok(m) => (Some(m), Some(hotkey.id())),
                Err(e) => {
                    error = Some(format!("Raccourci global indisponible : {e}"));
                    (None, None)
                }
            };
        let settings = Settings::load();
        let mut editor = Editor::default();
        let [r, g, b, a] = settings.default_color;
        editor.set_defaults(
            egui::Color32::from_rgba_unmultiplied(r, g, b, a),
            settings.default_stroke,
        );
        Self {
            stage: Stage::Empty,
            editor,
            pending_capture: None,
            pending_mode: CaptureMode::Region,
            pending_delay_ms: 400,
            error,
            status: None,
            selection: None,
            tray,
            _hotkey_manager: manager,
            hotkey_id,
            keep_annotations: false,
            quitting: false,
            first_frame: true,
            positioned: false,
            settings,
            settings_open: false,
            settings_draft: None,
            legend_open: false,
            ai_rx: None,
            ai_busy: None,
            ai_prompt: String::new(),
            window_picker: None,
            doc_name: String::new(),
        }
    }

    // ----- Petits utilitaires partagés par les sous-modules -------------------

    fn save_dir(&self) -> PathBuf {
        self.settings
            .save_dir
            .clone()
            .unwrap_or_else(crate::infra::export::default_save_dir)
    }

    fn update_title(&self, ctx: &egui::Context) {
        let title = if self.doc_name.is_empty() {
            "GEVSHOT".to_string()
        } else {
            format!("GEVSHOT — {}", self.doc_name)
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }

    fn set_status(&mut self, msg: impl Into<String>) {
        self.status = Some(msg.into());
        self.error = None;
    }

    fn set_error(&mut self, msg: impl Into<String>) {
        self.error = Some(msg.into());
        self.status = None;
    }

    /// Recadre l'image éditée à `rect` (coordonnées image) et translate
    /// les annotations pour qu'elles restent alignées sur le contenu.
    fn apply_crop(&mut self, ctx: &egui::Context, rect: egui::Rect) {
        let Stage::Editing { capture } = &mut self.stage else {
            return;
        };
        let (iw, ih) = (capture.image.width(), capture.image.height());
        let x = (rect.min.x.max(0.0) as u32).min(iw.saturating_sub(1));
        let y = (rect.min.y.max(0.0) as u32).min(ih.saturating_sub(1));
        let w = ((rect.width().round() as u32).max(1)).min(iw - x);
        let h = ((rect.height().round() as u32).max(1)).min(ih - y);
        if x == 0 && y == 0 && w == iw && h == ih {
            return;
        }
        let image = image::imageops::crop_imm(&capture.image, x, y, w, h).to_image();
        *capture = Capture::from_image(ctx, image);
        let removed = self.editor.translate_after_crop(
            -egui::vec2(x as f32, y as f32),
            egui::vec2(w as f32, h as f32),
        );
        let mut status = format!("Image recadrée : {w} × {h} px");
        if removed > 0 {
            status.push_str(&format!(" — {removed} annotation(s) hors zone supprimée(s)"));
        }
        self.set_status(status);
    }
}

impl eframe::App for GevshotApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Réveil périodique : les événements de la barre système et du
        // raccourci global arrivent hors du flux d'événements de la fenêtre,
        // et `logic` continue d'être appelé même fenêtre masquée.
        ctx.request_repaint_after(Duration::from_millis(150));

        if self.first_frame {
            self.first_frame = false;
            // Barre de titre claire, même si Windows est en thème sombre.
            ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Light));
            // Sans barre système, une fenêtre invisible serait inaccessible.
            if self.tray.is_none() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            }
        }

        // Taille par défaut : 6/8 de la largeur et 2/3 de la hauteur de
        // l'écran, fenêtre centrée. Fait dès que la taille du moniteur est
        // connue, hors sélection plein écran pour ne pas gêner l'overlay.
        if !self.positioned
            && !matches!(self.stage, Stage::Selecting { .. })
            && let Some(monitor) = ctx.input(|i| i.viewport().monitor_size)
            && monitor.x > 0.0
            && monitor.y > 0.0
        {
            let h = monitor.y * 2.0 / 3.0;
            let w = monitor.x * 6.0 / 8.0;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(w, h)));
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                (monitor.x - w) / 2.0,
                (monitor.y - h) / 2.0,
            )));
            self.positioned = true;
        }

        self.poll_integrations(ctx);
        self.poll_ai();

        // Fermer la fenêtre la masque vers la barre système au lieu de quitter.
        if self.tray.is_some() && !self.quitting && ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        self.poll_pending_capture(ctx);
    }

    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();

        if matches!(self.stage, Stage::Selecting { .. }) {
            self.region_select_ui(&ctx, root);
            return;
        }

        self.toolbar_ui(&ctx, root);
        self.shortcuts(&ctx);
        self.status_bar_ui(root);
        self.legend_panel_ui(root);

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(theme::BG_CANVAS))
            .show(root, |ui| match &self.stage {
                Stage::Editing { capture } => {
                    let size = capture.size();
                    self.editor.canvas(ui, &capture.texture, size);
                }
                _ => {
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(ui.available_height() / 2.0 - 30.0);
                            ui.label(
                                egui::RichText::new("Aucune capture")
                                    .size(17.0)
                                    .color(theme::TEXT_WEAK),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                egui::RichText::new(
                                    "Ctrl+Shift+S, le bouton caméra ou la barre système",
                                )
                                .size(13.0)
                                .color(theme::TEXT_WEAK),
                            );
                        });
                    });
                }
            });

        if let Some(rect) = self.editor.take_crop_request() {
            self.apply_crop(&ctx, rect);
        }

        self.window_picker_ui(&ctx);

        if self.settings_open {
            self.show_settings_window(&ctx);
        }
    }
}
