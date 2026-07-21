//! Couche application : l'état global (`StudioApp`), le cycle de vie eframe
//! et la composition des panneaux (barres, étapes, scène, inspecteur).

mod mode_bar;
mod stage_doc;
mod status_bar;
mod steps_panel;
mod title_bar;

use std::collections::HashMap;
use std::path::PathBuf;

use eframe::egui;
use gev_core::document::{self, Document, Step};
use gev_core::fonts::FontSet;
use gev_ui::editor::Tool;
use gev_ui::theme;

use crate::infra::{config::Settings, files};
use crate::license::License;
use crate::ui::toasts::Toasts;

/// Mode d'assemblage du guide, choisi dans la barre de modes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Document,
    Slides,
    Video,
}

impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Mode::Document => "Document",
            Mode::Slides => "Slides",
            Mode::Video => "Vidéo",
        }
    }
}

pub struct StudioApp {
    // Document.
    doc: Document,
    path: Option<PathBuf>,
    dirty: bool,
    selected: usize,
    // Édition.
    mode: Mode,
    tool: Tool,
    zoom_pct: u32,
    // Services.
    license: License,
    settings: Settings,
    toasts: Toasts,
    fonts: FontSet,
    /// Aperçus aplatis par index d'étape ; vidé à chaque mutation des étapes.
    textures: HashMap<usize, egui::TextureHandle>,
    first_frame: bool,
}

fn new_document() -> Document {
    Document {
        title: "Nouveau guide".to_string(),
        steps: Vec::new(),
        voice_id: String::new(),
    }
}

impl StudioApp {
    pub fn new(open_path: Option<PathBuf>) -> Self {
        let settings = Settings::load();
        let mut app = Self {
            doc: new_document(),
            path: None,
            dirty: false,
            selected: 0,
            mode: Mode::Document,
            tool: Tool::Select,
            zoom_pct: 100,
            license: License::load(),
            settings,
            toasts: Toasts::default(),
            fonts: FontSet::embedded(),
            textures: HashMap::new(),
            first_frame: true,
        };
        if let Some(path) = open_path {
            app.open_from(path);
        }
        app
    }

    // ----- Accès -----------------------------------------------------------

    pub(super) fn selected_step(&self) -> Option<&Step> {
        self.doc.steps.get(self.selected)
    }

    pub(super) fn is_pro(&self) -> bool {
        self.license.is_pro()
    }

    /// Aperçu aplati (image + annotations) d'une étape, mis en cache.
    pub(super) fn texture_for(
        &mut self,
        ctx: &egui::Context,
        index: usize,
    ) -> Option<egui::TextureHandle> {
        if let Some(tex) = self.textures.get(&index) {
            return Some(tex.clone());
        }
        let step = self.doc.steps.get(index)?;
        let flat = gev_core::render::flatten(&step.image, &step.annotations, &self.fonts);
        let (w, h) = flat.dimensions();
        let image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &flat);
        let tex = ctx.load_texture(
            format!("step_{index}"),
            image,
            egui::TextureOptions::LINEAR,
        );
        self.textures.insert(index, tex.clone());
        Some(tex)
    }

    // ----- Mutations du document ------------------------------------------

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// À appeler après toute mutation de la liste d'étapes : les aperçus
    /// sont indexés par position, on repart de zéro.
    fn invalidate_previews(&mut self) {
        self.textures.clear();
    }

    pub(super) fn select(&mut self, index: usize) {
        if index < self.doc.steps.len() {
            self.selected = index;
        }
    }

    /// Insère des étapes après la sélection et sélectionne la dernière.
    pub(super) fn insert_steps(&mut self, steps: Vec<Step>, errors: Vec<String>) {
        for err in &errors {
            self.toasts.push(err.clone());
        }
        if steps.is_empty() {
            return;
        }
        let count = steps.len();
        let at = (self.selected + 1).min(self.doc.steps.len());
        for (i, step) in steps.into_iter().enumerate() {
            self.doc.steps.insert(at + i, step);
        }
        self.selected = at + count - 1;
        self.mark_dirty();
        self.invalidate_previews();
        self.toasts.push(if count == 1 {
            "Étape importée depuis une capture".to_string()
        } else {
            format!("{count} étapes importées")
        });
    }

    pub(super) fn import_captures(&mut self) {
        let (steps, errors) = files::import_steps(self.settings.last_dir.as_deref());
        if let Some(first) = steps.first() {
            let _ = first; // le dossier est retenu via le premier chemin du dialogue
        }
        self.insert_steps(steps, errors);
    }

    pub(super) fn move_selected(&mut self, dir: isize) {
        let i = self.selected;
        let j = i as isize + dir;
        if j < 0 || j as usize >= self.doc.steps.len() {
            return;
        }
        self.doc.steps.swap(i, j as usize);
        self.selected = j as usize;
        self.mark_dirty();
        self.invalidate_previews();
    }

    pub(super) fn duplicate_selected(&mut self) {
        let Some(step) = self.doc.steps.get(self.selected) else {
            return;
        };
        let copy = step.clone();
        self.doc.steps.insert(self.selected + 1, copy);
        self.selected += 1;
        self.mark_dirty();
        self.invalidate_previews();
        self.toasts.push("Étape dupliquée");
    }

    pub(super) fn delete_selected(&mut self) {
        if self.selected >= self.doc.steps.len() {
            return;
        }
        self.doc.steps.remove(self.selected);
        if self.selected >= self.doc.steps.len() && self.selected > 0 {
            self.selected -= 1;
        }
        self.mark_dirty();
        self.invalidate_previews();
        self.toasts.push("Étape supprimée");
    }

    // ----- Fichier ---------------------------------------------------------

    pub(super) fn new_doc(&mut self) {
        self.doc = new_document();
        self.path = None;
        self.dirty = false;
        self.selected = 0;
        self.invalidate_previews();
    }

    pub(super) fn open(&mut self) {
        let Some((path, result)) = files::open_dialog(self.settings.last_dir.as_deref()) else {
            return;
        };
        match result {
            Ok(doc) => {
                self.settings.remember_dir(&path);
                self.doc = doc;
                self.path = Some(path);
                self.dirty = false;
                self.selected = 0;
                self.invalidate_previews();
            }
            Err(e) => self.toasts.push(format!("Ouverture impossible : {e}")),
        }
    }

    fn open_from(&mut self, path: PathBuf) {
        match document::load(&path) {
            Ok(doc) => {
                self.doc = doc;
                self.path = Some(path);
                self.dirty = false;
                self.selected = 0;
                self.invalidate_previews();
            }
            Err(e) => self.toasts.push(format!("Ouverture impossible : {e}")),
        }
    }

    pub(super) fn save(&mut self) {
        match &self.path {
            Some(path) => {
                let path = path.clone();
                self.save_to(path);
            }
            None => self.save_as(),
        }
    }

    pub(super) fn save_as(&mut self) {
        let Some(path) = files::save_dialog(&self.doc.title, self.settings.last_dir.as_deref())
        else {
            return;
        };
        self.save_to(path);
    }

    fn save_to(&mut self, path: PathBuf) {
        match document::save(&path, &self.doc) {
            Ok(()) => {
                self.settings.remember_dir(&path);
                self.path = Some(path);
                self.dirty = false;
                self.toasts.push("Guide enregistré");
            }
            Err(e) => self.toasts.push(format!("Enregistrement impossible : {e}")),
        }
    }

    // ----- Divers ----------------------------------------------------------

    fn update_title(&self, ctx: &egui::Context) {
        let star = if self.dirty { " •" } else { "" };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "GevShot Studio — {}{star}",
            self.doc.title
        )));
    }

    fn shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let save = ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S));
        if save {
            self.save();
        }
    }

    /// Import par glisser-déposer de fichiers sur la fenêtre.
    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .filter(|p| files::importable(p))
                .collect()
        });
        if !dropped.is_empty() {
            let (steps, errors) = files::load_steps(&dropped);
            self.insert_steps(steps, errors);
        }
    }
}

impl eframe::App for StudioApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.first_frame {
            self.first_frame = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::SetTheme(egui::SystemTheme::Light));
        }
        self.update_title(ctx);
        self.shortcuts(ctx);
        self.handle_dropped_files(ctx);
    }

    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();

        self.title_bar_ui(root);
        self.mode_bar_ui(root);
        self.status_bar_ui(root);
        self.steps_panel_ui(root);

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(theme::BG_APP))
            .show(root, |ui| {
                self.stage_ui(&ctx, ui);
            });

        self.toasts.ui(&ctx);
    }
}

impl StudioApp {
    /// Scène centrale selon le mode courant.
    fn stage_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        stage_doc::dotted_background(ui);
        if self.doc.steps.is_empty() {
            self.empty_state_ui(ui);
            return;
        }
        match self.mode {
            Mode::Document => self.stage_doc_ui(ctx, ui),
            // Scènes Slides et Vidéo : jalon M2.
            Mode::Slides | Mode::Video => {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(format!("Aperçu {} — bientôt", self.mode.label()))
                            .size(15.0)
                            .color(theme::TEXT_WEAK),
                    );
                });
            }
        }
    }

    /// État vide : invite à importer les premières captures.
    fn empty_state_ui(&mut self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() / 2.0 - 70.0);
                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(40.0, 40.0), egui::Sense::hover());
                crate::ui::icons::icon_image_plus(ui.painter(), rect.center(), theme::TEXT_WEAK);
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("Commencez votre guide")
                        .size(17.0)
                        .color(theme::TEXT),
                );
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(
                        "Importez des captures PNG/JPEG ou des projets .gev de GEVSHOT",
                    )
                    .size(13.0)
                    .color(theme::TEXT_WEAK),
                );
                ui.add_space(14.0);
                if gev_ui::widgets::pill_button(ui, "Importer des captures", true).clicked() {
                    self.import_captures();
                }
            });
        });
    }
}
