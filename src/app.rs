use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui;
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};

use crate::config::Settings;
use crate::editor::Editor;
use crate::settings_ui::{self, SettingsAction, SettingsDraft};
use crate::theme;
use crate::tray::{Tray, TrayCommand};
use crate::widgets;

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
    ai_rx: Option<std::sync::mpsc::Receiver<crate::ai::AiResult>>,
    /// Message affiché pendant un appel IA.
    ai_busy: Option<String>,
    /// Instruction saisie pour « Annoter par IA ».
    ai_prompt: String,
    /// Sélecteur de fenêtre à capturer (liste ouverte).
    window_picker: Option<Vec<crate::capture::WindowInfo>>,
    /// Nom du document courant, affiché dans le titre de la fenêtre.
    doc_name: String,
}

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
        if !self.positioned && !matches!(self.stage, Stage::Selecting { .. })
            && let Some(monitor) = ctx.input(|i| i.viewport().monitor_size)
                && monitor.x > 0.0 && monitor.y > 0.0 {
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

        self.window_picker_ui(&ctx);

        if self.settings_open {
            self.show_settings_window(&ctx);
        }
    }
}

impl GevshotApp {
    // ----- Barre d'outils ----------------------------------------------------

    fn toolbar_ui(&mut self, ctx: &egui::Context, root: &mut egui::Ui) {
        let editing = matches!(self.stage, Stage::Editing { .. });
        egui::Panel::top(egui::Id::new("toolbar")).show(root, |ui| {
            ui.add_space(6.0);

            // Une seule rangée : les groupes qui ne tiennent pas se replient
            // dans un menu « ⋯ » en bout de barre. Largeurs estimées.
            const W_FIXED: f32 = 92.0; // bouton capture + menu de modes + marges
            const W_TOOLS: f32 = 240.0;
            const W_COLORS: f32 = 155.0;
            const W_STROKES: f32 = 145.0;
            const W_UNDO: f32 = 85.0;
            const W_MORE: f32 = 40.0;
            let w_actions = if editing { 415.0 } else { 130.0 };
            let w_options = if editing {
                self.editor.options_group_width()
            } else {
                0.0
            };

            // Par priorité décroissante : le premier qui ne tient plus part
            // dans « ⋯ », ainsi que tous les suivants.
            // [couleurs, épaisseurs, undo/redo, options, actions]
            let groups = [
                (editing, W_COLORS),
                (editing, W_STROKES),
                (editing, W_UNDO),
                (editing && w_options > 0.0, w_options),
                (true, w_actions),
            ];
            let base = ui.available_width() - W_FIXED - if editing { W_TOOLS } else { 0.0 };
            let total: f32 = groups.iter().filter(|(v, _)| *v).map(|(_, w)| w).sum();
            let mut show = [true; 5];
            if total > base {
                let budget = base - W_MORE;
                let mut used = 0.0;
                let mut fits = true;
                for (i, (visible, w)) in groups.iter().enumerate() {
                    if !visible {
                        continue;
                    }
                    if fits && used + w <= budget {
                        used += w;
                    } else {
                        fits = false;
                        show[i] = false;
                    }
                }
            }
            let overflowing = groups
                .iter()
                .zip(show)
                .any(|((visible, _), shown)| *visible && !shown);

            ui.horizontal(|ui| {
                ui.add_space(4.0);
                self.capture_button_ui(ctx, ui);
                if editing {
                    ui.separator();
                    self.editor.tools_group_ui(ui);
                    if show[0] {
                        ui.separator();
                        self.editor.colors_group_ui(ui);
                    }
                    if show[1] {
                        ui.separator();
                        self.editor.strokes_group_ui(ui);
                    }
                    if show[3] && w_options > 0.0 {
                        ui.separator();
                        self.editor.options_group_ui(ui);
                    }
                    if show[2] {
                        ui.separator();
                        self.editor.undo_redo_group_ui(ui);
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(4.0);
                    if overflowing {
                        let more = widgets::icon_button(ui, false, widgets::icon_dots);
                        egui::Popup::menu(&more).show(|ui| {
                            self.overflow_menu_ui(ctx, ui, editing, show);
                        });
                    }
                    if show[4] {
                        self.action_buttons_ui(ctx, ui, editing);
                    }
                });
            });
            ui.add_space(6.0);
        });
    }

    /// Contenu du menu « ⋯ » : les groupes repliés.
    fn overflow_menu_ui(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        editing: bool,
        show: [bool; 5],
    ) {
        ui.set_min_width(230.0);
        if editing {
            if !show[0] {
                ui.horizontal(|ui| self.editor.colors_group_ui(ui));
            }
            if !show[1] {
                ui.horizontal(|ui| self.editor.strokes_group_ui(ui));
            }
            if !show[3] && self.editor.options_group_width() > 0.0 {
                ui.horizontal_wrapped(|ui| self.editor.options_group_ui(ui));
            }
            if !show[2] {
                ui.horizontal(|ui| self.editor.undo_redo_group_ui(ui));
            }
            if !show[4] && (!show[0] || !show[1] || !show[2] || !show[3]) {
                ui.separator();
            }
        }
        if !show[4] {
            if editing {
                if ui.button("Copier (Ctrl+C)").clicked() {
                    ui.close();
                    self.copy_to_clipboard();
                }
                if ui.button("Sauver (Ctrl+S)").clicked() {
                    ui.close();
                    self.save_auto(ctx);
                }
                ui.separator();
                if ui.button("Sauver le projet…").clicked() {
                    ui.close();
                    self.save_project();
                }
                if ui.button("Recapturer le fond").clicked() {
                    ui.close();
                    self.keep_annotations = true;
                    self.start_capture(ctx);
                }
            }
            if ui.button("Ouvrir un projet…").clicked() {
                ui.close();
                self.open_project(ctx);
            }
            ui.separator();
            if editing && ui.button("Panneau légende").clicked() {
                ui.close();
                self.legend_open = !self.legend_open;
            }
            if ui.button("Paramètres…").clicked() {
                ui.close();
                self.settings_open = true;
            }
        }
    }

    /// Panneau latéral : légende des pastilles + export Markdown.
    fn legend_panel_ui(&mut self, root: &mut egui::Ui) {
        if !self.legend_open || !matches!(self.stage, Stage::Editing { .. }) {
            return;
        }
        egui::Panel::right(egui::Id::new("legend"))
            .default_size(300.0)
            .show(root, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("Légende").size(16.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if widgets::icon_button(ui, false, widgets::icon_close)
                            .on_hover_text("Fermer")
                            .clicked()
                        {
                            self.legend_open = false;
                        }
                    });
                });
                ui.separator();
                self.editor.legend_ui(ui);
                if self.editor.has_badges() {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if widgets::pill_button(ui, "Copier en Markdown", false)
                            .on_hover_text(
                                "1. …\u{a0} 2. …\u{a0} — à coller sous l'image dans la doc",
                            )
                            .clicked()
                        {
                            match crate::export::copy_text(&self.editor.legend_markdown()) {
                                Ok(()) => self.set_status("Légende copiée en Markdown"),
                                Err(e) => self.set_error(e),
                            }
                        }
                        let can_ai = !self.settings.api_key.is_empty() && self.ai_rx.is_none();
                        let generate = ui
                            .add_enabled(can_ai, egui::Button::new("Générer par IA"))
                            .on_hover_text(
                                "Rédige le texte de chaque étape à partir de l'image annotée",
                            );
                        if self.settings.api_key.is_empty() {
                            generate.on_disabled_hover_text(
                                "Configurez la clé API dans Paramètres → IA",
                            );
                        } else if generate.clicked() {
                            self.start_ai_legend();
                        }
                    });
                }
            });
    }

    fn capture_button_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        if widgets::icon_button(ui, false, widgets::icon_camera)
            .on_hover_text("Capturer une région (Ctrl+Shift+S)")
            .clicked()
        {
            self.start_capture(ctx);
        }
        let modes = widgets::icon_button(ui, false, widgets::icon_chevron_down)
            .on_hover_text("Modes de capture");
        egui::Popup::menu(&modes).show(|ui| {
            ui.set_min_width(180.0);
            if ui.button("Région (Ctrl+Shift+S)").clicked() {
                ui.close();
                self.start_capture(ctx);
            }
            if ui.button("Région dans 3 s").clicked() {
                ui.close();
                self.start_capture_mode(ctx, CaptureMode::Region, 3000);
            }
            if ui.button("Écran entier").clicked() {
                ui.close();
                self.start_capture_mode(ctx, CaptureMode::FullScreen, 0);
            }
            if ui.button("Fenêtre active").clicked() {
                ui.close();
                self.start_capture_mode(ctx, CaptureMode::Window, 0);
            }
            if ui.button("Choisir une fenêtre…").clicked() {
                ui.close();
                self.open_window_picker();
            }
        });
    }

    fn open_window_picker(&mut self) {
        match crate::capture::list_windows() {
            Ok(windows) => {
                self.window_picker = Some(windows);
                self.error = None;
                self.status = None;
            }
            Err(e) => self.set_error(e),
        }
    }

    /// Fenêtre modale : liste des fenêtres capturables.
    fn window_picker_ui(&mut self, ctx: &egui::Context) {
        let Some(windows) = self.window_picker.clone() else {
            return;
        };
        let mut open = true;
        let mut picked: Option<u32> = None;
        egui::Window::new("Choisir une fenêtre à capturer")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size([440.0, 380.0])
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for w in &windows {
                            let label = if w.app_name.is_empty() {
                                w.title.clone()
                            } else {
                                format!("{} — {}", w.app_name, w.title)
                            };
                            if ui
                                .add(
                                    egui::Button::new(label)
                                        .wrap_mode(egui::TextWrapMode::Truncate)
                                        .min_size(egui::vec2(ui.available_width(), 30.0)),
                                )
                                .clicked()
                            {
                                picked = Some(w.id);
                            }
                        }
                    });
                ui.add_space(6.0);
                ui.weak("La fenêtre choisie est capturée telle qu'elle apparaît à l'écran.");
            });
        if let Some(id) = picked {
            self.window_picker = None;
            self.start_capture_mode(ctx, CaptureMode::WindowById(id), 0);
        } else if !open {
            self.window_picker = None;
        }
    }

    /// Boutons d'action, posés de droite à gauche : ⚙, Projet, Sauver, Copier.
    fn action_buttons_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, editing: bool) {
        ui.add_space(4.0);
        if widgets::icon_button(ui, false, widgets::icon_gear)
            .on_hover_text("Paramètres")
            .clicked()
        {
            self.settings_open = true;
        }
        if editing
            && widgets::icon_button(ui, self.legend_open, widgets::icon_menu_lines)
                .on_hover_text("Panneau légende — un texte par pastille, export Markdown")
                .clicked()
        {
            self.legend_open = !self.legend_open;
        }
        if editing {
            let sparkle = widgets::icon_button(ui, self.ai_rx.is_some(), widgets::icon_sparkle)
                .on_hover_text("Annoter par IA — décrivez ce qu'il faut annoter");
            egui::Popup::menu(&sparkle)
                .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
                .show(|ui| {
                    ui.set_min_width(300.0);
                    ui.label("Instruction :");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.ai_prompt)
                            .desired_rows(3)
                            .desired_width(f32::INFINITY)
                            .hint_text(
                                "Ex. : entoure le bouton Enregistrer et numérote \
                                 les champs du formulaire",
                            ),
                    );
                    if self.settings.api_key.is_empty() {
                        ui.weak("Configurez la clé API dans Paramètres → IA.");
                    }
                    let ready = !self.settings.api_key.is_empty()
                        && !self.ai_prompt.trim().is_empty()
                        && self.ai_rx.is_none();
                    if ui
                        .add_enabled(ready, egui::Button::new("Annoter"))
                        .clicked()
                    {
                        ui.close();
                        self.start_ai_annotate();
                    }
                });
        }
        ui.menu_button("Projet", |ui| {
            if ui.button("Ouvrir un projet…").clicked() {
                ui.close();
                self.open_project(ctx);
            }
            if ui
                .add_enabled(editing, egui::Button::new("Sauver le projet…"))
                .clicked()
            {
                ui.close();
                self.save_project();
            }
            if ui
                .add_enabled(editing, egui::Button::new("Recapturer le fond"))
                .on_hover_text(
                    "Nouvelle capture en conservant les annotations — \
                     recadrez la même zone",
                )
                .clicked()
            {
                ui.close();
                self.keep_annotations = true;
                self.start_capture(ctx);
            }
        });
        if editing {
            if widgets::pill_button(ui, "Sauver", true)
                .on_hover_text(format!("Ctrl+S — PNG dans {}", self.save_dir().display()))
                .clicked()
            {
                self.save_auto(ctx);
            }
            if widgets::pill_button(ui, "Copier", false)
                .on_hover_text("Ctrl+C — copie l'image annotée")
                .clicked()
            {
                self.copy_to_clipboard();
            }
        }
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        if !matches!(self.stage, Stage::Editing { .. }) || ctx.egui_wants_keyboard_input() {
            return;
        }
        let (copy, save) = ctx.input(|i| {
            (
                i.modifiers.ctrl && i.key_pressed(egui::Key::C),
                i.modifiers.ctrl && i.key_pressed(egui::Key::S),
            )
        });
        if copy {
            self.copy_to_clipboard();
        }
        if save {
            self.save_auto(ctx);
        }
    }

    // ----- Barre d'état --------------------------------------------------------

    fn status_bar_ui(&mut self, root: &mut egui::Ui) {
        egui::Panel::bottom(egui::Id::new("status")).show(root, |ui| {
            let (rect, _) = ui
                .allocate_exact_size(egui::vec2(ui.available_width(), 26.0), egui::Sense::hover());
            let p = ui.painter();
            let font = egui::FontId::proportional(13.0);

            // Textes latéraux, mesurés pour réserver l'espace du centre.
            let mut left_end = rect.min.x + 10.0;
            let mut right_start = rect.max.x - 10.0;
            if let Stage::Editing { capture } = &self.stage {
                let dims = format!(
                    "{} × {} px · {:.0} %",
                    capture.image.width(),
                    capture.image.height(),
                    self.editor.zoom() * 100.0
                );
                let r = p.text(
                    egui::pos2(rect.min.x + 10.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    dims,
                    font.clone(),
                    theme::TEXT_WEAK,
                );
                left_end = r.max.x;
                let count = self.editor.annotation_count();
                let r = p.text(
                    egui::pos2(rect.max.x - 10.0, rect.center().y),
                    egui::Align2::RIGHT_CENTER,
                    format!("{count} annotation{}", if count > 1 { "s" } else { "" }),
                    font.clone(),
                    theme::TEXT_WEAK,
                );
                right_start = r.min.x;
            }

            let (msg, color) = if let Some(e) = &self.error {
                (e.clone(), theme::DANGER)
            } else if let Some(busy) = &self.ai_busy {
                (busy.clone(), theme::ACCENT)
            } else if let Some(s) = &self.status {
                (s.clone(), theme::SUCCESS)
            } else if matches!(self.stage, Stage::Editing { .. }) {
                (self.editor.hint().to_string(), theme::TEXT_WEAK)
            } else {
                ("Ctrl+Shift+S pour capturer".to_string(), theme::TEXT_WEAK)
            };

            // Message central tronqué (…) s'il ne tient pas entre les côtés.
            let available = (right_start - left_end - 32.0).max(0.0);
            if available > 20.0 {
                let mut job = egui::text::LayoutJob::simple_singleline(msg, font, color);
                job.wrap = egui::text::TextWrapping::truncate_at_width(available);
                let galley = p.layout_job(job);
                let center_x = (left_end + right_start) / 2.0;
                let pos = egui::pos2(
                    center_x - galley.size().x / 2.0,
                    rect.center().y - galley.size().y / 2.0,
                );
                p.galley(pos, galley, color);
            }
        });
    }

    // ----- IA -----------------------------------------------------------------

    fn poll_ai(&mut self) {
        let Some(rx) = &self.ai_rx else { return };
        match rx.try_recv() {
            Ok(result) => {
                self.ai_rx = None;
                self.ai_busy = None;
                match result {
                    crate::ai::AiResult::Legend(steps) => {
                        let n = steps.len();
                        self.editor.set_badge_labels(&steps);
                        self.legend_open = true;
                        self.set_status(format!("Légende générée par IA ({n} étapes)"));
                    }
                    crate::ai::AiResult::Annotate(annotations) => {
                        let n = annotations.len();
                        self.editor.add_annotations(annotations);
                        self.set_status(format!("{n} annotation(s) générée(s) par IA"));
                    }
                    crate::ai::AiResult::Error(e) => self.set_error(format!("IA : {e}")),
                }
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.ai_rx = None;
                self.ai_busy = None;
                self.set_error("IA : le traitement a échoué");
            }
        }
    }

    fn start_ai_legend(&mut self) {
        let Stage::Editing { capture } = &self.stage else {
            return;
        };
        // L'image annotée : le modèle voit les pastilles et leurs numéros.
        let image = crate::export::flatten(&capture.image, self.editor.annotations());
        let numbers = self.editor.badge_numbers();
        let (tx, rx) = std::sync::mpsc::channel();
        crate::ai::spawn_legend(
            self.settings.api_key.clone(),
            self.settings.ai_model.clone(),
            image,
            numbers,
            tx,
        );
        self.ai_rx = Some(rx);
        self.ai_busy = Some("Génération de la légende par IA…".to_string());
        self.error = None;
        self.status = None;
    }

    fn start_ai_annotate(&mut self) {
        let Stage::Editing { capture } = &self.stage else {
            return;
        };
        let (tx, rx) = std::sync::mpsc::channel();
        crate::ai::spawn_annotate(
            self.settings.api_key.clone(),
            self.settings.ai_model.clone(),
            capture.image.clone(),
            self.ai_prompt.trim().to_string(),
            self.editor.ai_style(),
            tx,
        );
        self.ai_rx = Some(rx);
        self.ai_busy = Some("Annotation par IA en cours…".to_string());
        self.error = None;
        self.status = None;
    }

    // ----- Barre système et raccourci global --------------------------------

    fn poll_integrations(&mut self, ctx: &egui::Context) {
        if let Some(cmd) = self.tray.as_ref().and_then(Tray::poll) {
            match cmd {
                TrayCommand::Capture => self.start_capture(ctx),
                TrayCommand::CaptureDelayed => {
                    self.start_capture_mode(ctx, CaptureMode::Region, 3000);
                }
                TrayCommand::CaptureScreen => {
                    self.start_capture_mode(ctx, CaptureMode::FullScreen, 0);
                }
                TrayCommand::CaptureWindow => {
                    self.start_capture_mode(ctx, CaptureMode::Window, 0);
                }
                TrayCommand::PickWindow => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    self.open_window_picker();
                }
                TrayCommand::OpenEditor => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                TrayCommand::Quit => {
                    self.quitting = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
        if let Ok(ev) = GlobalHotKeyEvent::receiver().try_recv()
            && Some(ev.id()) == self.hotkey_id
                && ev.state() == HotKeyState::Pressed
                && self.pending_capture.is_none()
                && !matches!(self.stage, Stage::Selecting { .. })
            {
                self.start_capture(ctx);
            }
    }

    // ----- Capture ----------------------------------------------------------

    fn start_capture(&mut self, ctx: &egui::Context) {
        self.start_capture_mode(ctx, CaptureMode::Region, 0);
    }

    /// Lance une capture : masque la fenêtre, attend le délai (masquage +
    /// minuteur optionnel), puis prend la vue selon le mode.
    fn start_capture_mode(&mut self, ctx: &egui::Context, mode: CaptureMode, extra_delay_ms: u64) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        self.pending_mode = mode;
        self.pending_delay_ms = self.settings.capture_delay_ms + extra_delay_ms;
        self.pending_capture = Some(Instant::now());
        self.error = None;
        self.status = None;
    }

    fn poll_pending_capture(&mut self, ctx: &egui::Context) {
        let Some(started) = self.pending_capture else {
            return;
        };
        if started.elapsed() < Duration::from_millis(self.pending_delay_ms) {
            ctx.request_repaint_after(Duration::from_millis(50));
            return;
        }
        self.pending_capture = None;
        match self.pending_mode {
            CaptureMode::Region => match crate::capture::capture_primary() {
                Ok(image) => {
                    let full = Capture::from_image(ctx, image);
                    let prev = match std::mem::replace(&mut self.stage, Stage::Empty) {
                        Stage::Editing { capture } => Some(capture),
                        _ => None,
                    };
                    self.stage = Stage::Selecting { full, prev };
                    self.selection = None;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                }
                Err(err) => self.error = Some(err),
            },
            CaptureMode::FullScreen => match crate::capture::capture_primary() {
                Ok(image) => self.enter_editing(ctx, image),
                Err(err) => self.error = Some(err),
            },
            CaptureMode::Window => match crate::capture::capture_focused_window() {
                Ok(image) => self.enter_editing(ctx, image),
                Err(err) => self.error = Some(err),
            },
            CaptureMode::WindowById(id) => match crate::capture::capture_window_by_id(id) {
                Ok(image) => self.enter_editing(ctx, image),
                Err(err) => self.error = Some(err),
            },
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    /// Passe directement en édition avec l'image donnée (plein écran, fenêtre).
    fn enter_editing(&mut self, ctx: &egui::Context, image: image::RgbaImage) {
        let capture = Capture::from_image(ctx, image);
        self.stage = Stage::Editing { capture };
        if !std::mem::take(&mut self.keep_annotations) {
            self.editor.reset();
        }
        self.doc_name = crate::export::next_auto_name(&self.save_dir());
        self.update_title(ctx);
    }

    // ----- Sélection de région ----------------------------------------------

    fn region_select_ui(&mut self, ctx: &egui::Context, root: &mut egui::Ui) {
        let (escape, enter) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::Enter),
            )
        });
        if escape {
            self.cancel_selection(ctx);
            return;
        }
        if enter {
            self.commit_region(ctx, None);
            return;
        }

        let mut committed: Option<Option<egui::Rect>> = None;
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(root, |ui| {
                let Stage::Selecting { full, .. } = &self.stage else {
                    return;
                };
                let rect = ui.max_rect();
                let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
                let painter = ui.painter();
                painter.image(
                    full.texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                if let Some(p) = response.interact_pointer_pos() {
                    if response.drag_started() {
                        self.selection = Some((p, p));
                    } else if response.dragged()
                        && let Some((_, cur)) = &mut self.selection {
                            *cur = p;
                        }
                }

                let sel_rect = self.selection.map(|(a, b)| egui::Rect::from_two_pos(a, b));

                // Voile sombre autour de la sélection (ou partout si aucune).
                let veil = egui::Color32::from_black_alpha(120);
                match sel_rect {
                    Some(sel) => {
                        let sel = sel.intersect(rect);
                        for part in [
                            egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, sel.min.y)),
                            egui::Rect::from_min_max(egui::pos2(rect.min.x, sel.max.y), rect.max),
                            egui::Rect::from_min_max(
                                egui::pos2(rect.min.x, sel.min.y),
                                egui::pos2(sel.min.x, sel.max.y),
                            ),
                            egui::Rect::from_min_max(
                                egui::pos2(sel.max.x, sel.min.y),
                                egui::pos2(rect.max.x, sel.max.y),
                            ),
                        ] {
                            painter.rect_filled(part, 0.0, veil);
                        }
                        painter.rect_stroke(
                            sel,
                            egui::CornerRadius::ZERO,
                            egui::Stroke::new(1.5, egui::Color32::WHITE),
                            egui::StrokeKind::Outside,
                        );
                        // Dimensions en pixels image.
                        let scale = full.size().x / rect.width();
                        let dims = format!(
                            "{} × {}",
                            (sel.width() * scale).round() as u32,
                            (sel.height() * scale).round() as u32
                        );
                        painter.text(
                            sel.min + egui::vec2(0.0, -8.0),
                            egui::Align2::LEFT_BOTTOM,
                            dims,
                            egui::FontId::proportional(14.0),
                            egui::Color32::WHITE,
                        );
                    }
                    None => {
                        painter.rect_filled(rect, 0.0, veil);
                    }
                }

                draw_hint_pill(painter, rect);

                if response.drag_stopped()
                    && let Some((a, b)) = self.selection.take() {
                        let sel = egui::Rect::from_two_pos(a, b).intersect(rect);
                        if sel.width() >= 5.0 && sel.height() >= 5.0 {
                            // Conversion écran → pixels image.
                            let scale = full.size().x / rect.width();
                            let img_rect = egui::Rect::from_min_max(
                                ((sel.min - rect.min) * scale).to_pos2(),
                                ((sel.max - rect.min) * scale).to_pos2(),
                            );
                            committed = Some(Some(img_rect));
                        }
                    }

                response.on_hover_cursor(egui::CursorIcon::Crosshair);
            });

        if let Some(region) = committed {
            self.commit_region(ctx, region);
        }
    }

    fn cancel_selection(&mut self, ctx: &egui::Context) {
        if let Stage::Selecting { prev, .. } = std::mem::replace(&mut self.stage, Stage::Empty)
            && let Some(capture) = prev {
                // L'éditeur n'a pas été réinitialisé : la session reprend telle quelle.
                self.stage = Stage::Editing { capture };
            }
        self.keep_annotations = false;
        self.selection = None;
        restore_window(ctx);
    }

    /// Termine la sélection : `region` en pixels image, `None` = écran entier.
    fn commit_region(&mut self, ctx: &egui::Context, region: Option<egui::Rect>) {
        let Stage::Selecting { full, .. } = std::mem::replace(&mut self.stage, Stage::Empty) else {
            return;
        };
        let image = match region {
            Some(r) => {
                let (iw, ih) = (full.image.width(), full.image.height());
                let x = (r.min.x.max(0.0) as u32).min(iw.saturating_sub(1));
                let y = (r.min.y.max(0.0) as u32).min(ih.saturating_sub(1));
                let w = ((r.width() as u32).max(1)).min(iw - x);
                let h = ((r.height() as u32).max(1)).min(ih - y);
                image::imageops::crop_imm(&full.image, x, y, w, h).to_image()
            }
            None => full.image,
        };
        let capture = Capture::from_image(ctx, image);
        self.stage = Stage::Editing { capture };
        if !std::mem::take(&mut self.keep_annotations) {
            self.editor.reset();
        }
        self.selection = None;
        self.doc_name = crate::export::next_auto_name(&self.save_dir());
        self.update_title(ctx);
        restore_window(ctx);
    }

    // ----- Export -----------------------------------------------------------

    fn save_dir(&self) -> PathBuf {
        self.settings
            .save_dir
            .clone()
            .unwrap_or_else(crate::export::default_save_dir)
    }

    fn update_title(&self, ctx: &egui::Context) {
        let title = if self.doc_name.is_empty() {
            "GEVSHOT".to_string()
        } else {
            format!("GEVSHOT — {}", self.doc_name)
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }

    /// Image finale : capture + annotations aplaties.
    fn rendered_image(&self) -> Option<image::RgbaImage> {
        let Stage::Editing { capture } = &self.stage else {
            return None;
        };
        Some(crate::export::flatten(
            &capture.image,
            self.editor.annotations(),
        ))
    }

    fn copy_to_clipboard(&mut self) {
        let Some(img) = self.rendered_image() else {
            return;
        };
        match crate::export::copy_to_clipboard(&img) {
            Ok(()) => self.set_status("Copié dans le presse-papiers"),
            Err(e) => self.set_error(e),
        }
    }

    fn save_auto(&mut self, ctx: &egui::Context) {
        let Some(img) = self.rendered_image() else {
            return;
        };
        match crate::export::save_auto(&img, &self.save_dir()) {
            Ok(path) => {
                if let Some(stem) = path.file_stem() {
                    self.doc_name = stem.to_string_lossy().into_owned();
                    self.update_title(ctx);
                }
                if self.settings.copy_on_save {
                    match crate::export::copy_to_clipboard(&img) {
                        Ok(()) => {
                            self.set_status(format!("Sauvé + copié : {}", path.display()));
                        }
                        Err(e) => self.set_error(e),
                    }
                } else {
                    self.set_status(format!("Sauvé : {}", path.display()));
                }
            }
            Err(e) => self.set_error(e),
        }
    }

    #[allow(dead_code)]
    fn save_dialog(&mut self) {
        let Some(img) = self.rendered_image() else {
            return;
        };
        match crate::export::save_dialog(&img, &self.save_dir()) {
            Ok(Some(path)) => self.set_status(format!("Sauvé : {}", path.display())),
            Ok(None) => {}
            Err(e) => self.set_error(e),
        }
    }

    fn set_status(&mut self, msg: impl Into<String>) {
        self.status = Some(msg.into());
        self.error = None;
    }

    fn set_error(&mut self, msg: impl Into<String>) {
        self.error = Some(msg.into());
        self.status = None;
    }

    // ----- Projet .gev ----------------------------------------------------------

    fn save_project(&mut self) {
        let Stage::Editing { capture } = &self.stage else {
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Projet GEVSHOT", &[crate::project::EXTENSION])
            .set_file_name(format!(
                "{}.{}",
                if self.doc_name.is_empty() {
                    "capture"
                } else {
                    &self.doc_name
                },
                crate::project::EXTENSION
            ))
            .set_directory(self.save_dir())
            .save_file()
        else {
            return;
        };
        match crate::project::save(&path, &capture.image, self.editor.annotations()) {
            Ok(()) => self.set_status(format!("Projet sauvé : {}", path.display())),
            Err(e) => self.set_error(e),
        }
    }

    fn open_project(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Projet GEVSHOT", &[crate::project::EXTENSION])
            .set_directory(self.save_dir())
            .pick_file()
        else {
            return;
        };
        match crate::project::load(&path) {
            Ok((image, annotations)) => {
                let capture = Capture::from_image(ctx, image);
                self.stage = Stage::Editing { capture };
                self.editor.set_annotations(annotations);
                self.doc_name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                self.update_title(ctx);
                self.set_status(format!("Projet ouvert : {}", path.display()));
            }
            Err(e) => self.set_error(e),
        }
    }

    // ----- Paramètres ---------------------------------------------------------

    fn show_settings_window(&mut self, ctx: &egui::Context) {
        if self.settings_draft.is_none() {
            self.settings_draft = Some(SettingsDraft::new(&self.settings));
        }
        let mut action = SettingsAction::None;
        let mut close_requested = false;
        // Modale dans la fenêtre principale : fond assombri, Échap ou clic
        // à l'extérieur pour fermer.
        let modal = egui::Modal::new(egui::Id::new("settings_modal")).show(ctx, |ui| {
            let avail = ui.ctx().content_rect();
            ui.set_width((avail.width() - 120.0).clamp(520.0, 760.0));
            ui.set_height((avail.height() - 140.0).clamp(360.0, 520.0));
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("Paramètres").size(17.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if widgets::icon_button(ui, false, widgets::icon_close)
                        .on_hover_text("Fermer")
                        .clicked()
                    {
                        close_requested = true;
                    }
                });
            });
            ui.separator();
            if let Some(draft) = self.settings_draft.as_mut() {
                action = settings_ui::show(ui, draft);
            }
        });
        if modal.should_close() {
            close_requested = true;
        }
        match action {
            SettingsAction::Apply => {
                if let Some(draft) = self.settings_draft.take() {
                    self.settings = draft.settings;
                    let [r, g, b, a] = self.settings.default_color;
                    self.editor.set_defaults(
                        egui::Color32::from_rgba_unmultiplied(r, g, b, a),
                        self.settings.default_stroke,
                    );
                    if let Err(e) = self.settings.save() {
                        self.set_error(e);
                    } else {
                        self.set_status("Paramètres enregistrés");
                    }
                }
                self.settings_open = false;
            }
            SettingsAction::Cancel => {
                self.settings_draft = None;
                self.settings_open = false;
            }
            SettingsAction::None => {
                if close_requested {
                    self.settings_draft = None;
                    self.settings_open = false;
                }
            }
        }
    }
}

/// Pilule d'aide sombre, centrée en haut de l'écran pendant la sélection.
fn draw_hint_pill(painter: &egui::Painter, screen: egui::Rect) {
    use egui::{CornerRadius, FontId, Rect, Shape, Stroke, pos2, vec2};

    let galley = painter.layout_no_wrap(
        "Tracez un rectangle pour capturer une région".to_string(),
        FontId::proportional(15.0),
        egui::Color32::WHITE,
    );
    let key_galley = painter.layout_no_wrap(
        "Échap".to_string(),
        FontId::proportional(13.0),
        theme::PILL_KEY_TEXT,
    );

    let (pad, gap, icon_w, h) = (18.0, 12.0, 18.0, 46.0);
    let key_w = key_galley.size().x + 16.0;
    let w = pad + icon_w + gap + galley.size().x + gap + key_w + pad;
    let rect = Rect::from_center_size(pos2(screen.center().x, screen.min.y + 44.0), vec2(w, h));
    painter.rect_filled(rect, CornerRadius::same(23), theme::PILL_BG);

    // Icône : carré en pointillés avec une petite flèche vers le coin.
    let stroke = Stroke::new(1.5, theme::PILL_ICON);
    let ic = Rect::from_center_size(
        pos2(rect.min.x + pad + icon_w / 2.0, rect.center().y),
        vec2(15.0, 15.0),
    );
    let corners = [
        ic.left_top(),
        ic.right_top(),
        ic.right_bottom(),
        ic.left_bottom(),
        ic.left_top(),
    ];
    for s in Shape::dashed_line(&corners, stroke, 2.6, 2.2) {
        painter.add(s);
    }
    let tip = ic.center() + vec2(-2.0, -2.0);
    painter.line_segment([ic.center() + vec2(3.5, 3.5), tip], stroke);
    painter.line_segment([tip, tip + vec2(3.5, 0.0)], stroke);
    painter.line_segment([tip, tip + vec2(0.0, 3.5)], stroke);

    // Texte.
    painter.galley(
        pos2(
            rect.min.x + pad + icon_w + gap,
            rect.center().y - galley.size().y / 2.0,
        ),
        galley,
        egui::Color32::WHITE,
    );

    // Touche Échap.
    let key_rect = Rect::from_min_size(
        pos2(rect.max.x - pad - key_w, rect.center().y - 13.0),
        vec2(key_w, 26.0),
    );
    painter.rect_stroke(
        key_rect,
        CornerRadius::same(7),
        Stroke::new(1.0, theme::PILL_KEY_BORDER),
        egui::StrokeKind::Middle,
    );
    let key_size = key_galley.size();
    painter.galley(
        key_rect.center() - key_size / 2.0,
        key_galley,
        theme::PILL_KEY_TEXT,
    );
}

fn restore_window(ctx: &egui::Context) {
    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
    ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(true));
    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
}
