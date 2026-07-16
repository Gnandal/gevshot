//! Barre d'outils : bouton capture + modes, groupes de l'éditeur avec
//! repli « ⋯ », boutons d'action, raccourcis clavier de l'édition.

use eframe::egui;

use super::{CaptureMode, GevshotApp, Stage};
use crate::ui::widgets;

impl GevshotApp {
    pub(super) fn toolbar_ui(&mut self, ctx: &egui::Context, root: &mut egui::Ui) {
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

    /// Bouton caméra + menu des modes de capture.
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

    /// Boutons d'action, posés de droite à gauche : ⚙, légende, IA,
    /// Projet, Sauver, Copier.
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

    /// Raccourcis clavier de l'édition (Ctrl+C, Ctrl+S).
    pub(super) fn shortcuts(&mut self, ctx: &egui::Context) {
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
}
