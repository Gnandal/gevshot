//! Interface GEVSTUDIO : ouvrir/enregistrer un `.gevd`, gérer la timeline des
//! étapes (ajout, réordonnancement, suppression), éditer titre et légende, et
//! exporter le guide en PDF.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use eframe::egui;
use gev_core::document::{self, Document, Step};
use gev_core::fonts::FontSet;
use gev_core::{project, render};

use crate::{pdf, theme};

/// Lance l'interface. Un `.gevd` optionnel est ouvert au démarrage.
pub fn launch(open: Option<PathBuf>) -> ExitCode {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 760.0])
            .with_min_inner_size([760.0, 520.0])
            .with_title("GEVSTUDIO"),
        ..Default::default()
    };
    let result = eframe::run_native(
        "GEVSTUDIO",
        options,
        Box::new(move |cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(GevstudioApp::new(open)))
        }),
    );
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erreur interface : {e}");
            ExitCode::FAILURE
        }
    }
}

struct GevstudioApp {
    doc: Document,
    path: Option<PathBuf>,
    selected: usize,
    fonts: FontSet,
    /// Aperçu aplati de l'étape courante, et l'indice qu'il représente.
    preview: Option<egui::TextureHandle>,
    preview_for: Option<usize>,
    status: String,
}

impl GevstudioApp {
    fn new(open: Option<PathBuf>) -> Self {
        let mut app = Self {
            doc: Document {
                title: String::new(),
                steps: Vec::new(),
            },
            path: None,
            selected: 0,
            fonts: FontSet::embedded(),
            preview: None,
            preview_for: None,
            status: "Nouveau document. Ajoutez une étape ou ouvrez un .gevd.".into(),
        };
        if let Some(p) = open {
            app.open_path(&p);
        }
        app
    }

    // ----- Fichiers ----------------------------------------------------------

    fn open_path(&mut self, path: &Path) {
        match document::load(path) {
            Ok(doc) => {
                let n = doc.steps.len();
                self.doc = doc;
                self.path = Some(path.to_path_buf());
                self.selected = 0;
                self.invalidate_preview();
                self.status = format!("Ouvert : {} ({n} étape(s))", path.display());
            }
            Err(e) => self.status = format!("Ouverture impossible : {e}"),
        }
    }

    fn open_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Document GEVSTUDIO", &["gevd"])
            .pick_file()
        {
            self.open_path(&path);
        }
    }

    fn save(&mut self, force_dialog: bool) {
        let path = if force_dialog || self.path.is_none() {
            rfd::FileDialog::new()
                .add_filter("Document GEVSTUDIO", &["gevd"])
                .set_file_name("guide.gevd")
                .save_file()
        } else {
            self.path.clone()
        };
        let Some(path) = path else { return };
        match document::save(&path, &self.doc) {
            Ok(()) => {
                self.path = Some(path.clone());
                self.status = format!("Enregistré : {}", path.display());
            }
            Err(e) => self.status = format!("Enregistrement impossible : {e}"),
        }
    }

    fn export_pdf(&mut self) {
        if self.doc.steps.is_empty() {
            self.status = "Rien à exporter : le document est vide.".into();
            return;
        }
        let default = self
            .path
            .as_ref()
            .map(|p| p.with_extension("pdf"))
            .unwrap_or_else(|| PathBuf::from("guide.pdf"));
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            .set_file_name(default.file_name().and_then(|s| s.to_str()).unwrap_or("guide.pdf"))
            .save_file()
        else {
            return;
        };
        match pdf::build(&self.doc).and_then(|bytes| {
            std::fs::write(&path, &bytes).map_err(|e| format!("écriture impossible : {e}"))
        }) {
            Ok(()) => self.status = format!("PDF exporté : {}", path.display()),
            Err(e) => self.status = format!("Export impossible : {e}"),
        }
    }

    fn add_step_from_image(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg"])
            .pick_file()
        else {
            return;
        };
        match image::open(&path) {
            Ok(img) => {
                self.push_step(Step {
                    image: img.to_rgba8(),
                    annotations: Vec::new(),
                    title: String::new(),
                    caption: String::new(),
                });
                self.status = format!("Étape ajoutée depuis {}", path.display());
            }
            Err(e) => self.status = format!("Image illisible : {e}"),
        }
    }

    fn add_step_from_gev(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Capture GEVSHOT", &["gev"])
            .pick_file()
        else {
            return;
        };
        match project::load(&path) {
            Ok((image, annotations)) => {
                self.push_step(Step {
                    image,
                    annotations,
                    title: String::new(),
                    caption: String::new(),
                });
                self.status = format!("Étape ajoutée depuis {}", path.display());
            }
            Err(e) => self.status = format!("Capture illisible : {e}"),
        }
    }

    fn push_step(&mut self, step: Step) {
        self.doc.steps.push(step);
        self.selected = self.doc.steps.len() - 1;
        self.invalidate_preview();
    }

    // ----- Timeline ----------------------------------------------------------

    fn select(&mut self, i: usize) {
        if i != self.selected {
            self.selected = i;
            self.invalidate_preview();
        }
    }

    fn move_step(&mut self, i: usize, delta: isize) {
        let j = i as isize + delta;
        if j < 0 || j as usize >= self.doc.steps.len() {
            return;
        }
        let j = j as usize;
        self.doc.steps.swap(i, j);
        self.selected = j;
        self.invalidate_preview();
    }

    fn delete_step(&mut self, i: usize) {
        if i >= self.doc.steps.len() {
            return;
        }
        self.doc.steps.remove(i);
        self.selected = self.selected.min(self.doc.steps.len().saturating_sub(1));
        self.invalidate_preview();
    }

    // ----- Aperçu ------------------------------------------------------------

    fn invalidate_preview(&mut self) {
        self.preview = None;
        self.preview_for = None;
    }

    fn ensure_preview(&mut self, ctx: &egui::Context) {
        if self.preview.is_some() && self.preview_for == Some(self.selected) {
            return;
        }
        let Some(step) = self.doc.steps.get(self.selected) else {
            self.preview = None;
            self.preview_for = None;
            return;
        };
        let flat = render::flatten(&step.image, &step.annotations, &self.fonts);
        let size = [flat.width() as usize, flat.height() as usize];
        let color = egui::ColorImage::from_rgba_unmultiplied(size, flat.as_raw());
        let tex = ctx.load_texture("preview", color, egui::TextureOptions::LINEAR);
        self.preview = Some(tex);
        self.preview_for = Some(self.selected);
    }

    // ----- Rendu -------------------------------------------------------------

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Ouvrir…").clicked() {
                self.open_dialog();
            }
            if ui.button("Enregistrer").clicked() {
                self.save(false);
            }
            if ui.button("Enregistrer sous…").clicked() {
                self.save(true);
            }
            ui.separator();
            if ui.button("+ Image…").clicked() {
                self.add_step_from_image();
            }
            if ui.button("+ depuis .gev…").clicked() {
                self.add_step_from_gev();
            }
            ui.separator();
            if ui.button("Exporter PDF…").clicked() {
                self.export_pdf();
            }
            ui.separator();
            ui.label("Titre :");
            ui.add(
                egui::TextEdit::singleline(&mut self.doc.title)
                    .hint_text("Titre du guide")
                    .desired_width(240.0),
            );
        });
    }

    fn steps_panel(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.heading(format!("Étapes ({})", self.doc.steps.len()));
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let count = self.doc.steps.len();
            for i in 0..count {
                let label = {
                    let t = &self.doc.steps[i].title;
                    if t.is_empty() {
                        format!("{}. (sans titre)", i + 1)
                    } else {
                        format!("{}. {t}", i + 1)
                    }
                };
                let selected = i == self.selected;
                if ui.selectable_label(selected, label).clicked() {
                    self.select(i);
                }
                if selected {
                    ui.horizontal(|ui| {
                        if ui.small_button("Monter").clicked() {
                            self.move_step(i, -1);
                        }
                        if ui.small_button("Descendre").clicked() {
                            self.move_step(i, 1);
                        }
                        if ui.small_button("Supprimer").clicked() {
                            self.delete_step(i);
                        }
                    });
                }
            }
            if count == 0 {
                ui.weak("Aucune étape. Utilisez « ＋ Image… » ou « ＋ depuis .gev… ».");
            }
        });
    }

    fn central(&mut self, ui: &mut egui::Ui) {
        if self.doc.steps.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.weak("Ouvrez un document ou ajoutez une première étape.");
            });
            return;
        }
        let selected = self.selected;

        // Titre de l'étape.
        ui.horizontal(|ui| {
            ui.label("Titre de l'étape :");
            if let Some(step) = self.doc.steps.get_mut(selected) {
                ui.add(
                    egui::TextEdit::singleline(&mut step.title)
                        .hint_text("(optionnel)")
                        .desired_width(f32::INFINITY),
                );
            }
        });
        ui.add_space(6.0);

        // Légende, ancrée en bas ; l'aperçu occupe la place restante.
        egui::Panel::bottom(egui::Id::new("caption"))
            .frame(egui::Frame::default().inner_margin(egui::Margin::symmetric(0, 6)))
            .show(ui, |ui| {
                ui.label("Légende :");
                if let Some(step) = self.doc.steps.get_mut(selected) {
                    ui.add(
                        egui::TextEdit::multiline(&mut step.caption)
                            .hint_text("Texte affiché sous la capture")
                            .desired_width(f32::INFINITY)
                            .desired_rows(3),
                    );
                }
            });

        // Aperçu aplati, centré et mis à l'échelle.
        egui::Frame::default()
            .fill(theme::BG_CANVAS)
            .inner_margin(egui::Margin::same(8))
            .show(ui, |ui| {
                let avail = ui.available_size();
                ui.centered_and_justified(|ui| {
                    if let Some(tex) = &self.preview {
                        let sized = egui::load::SizedTexture::from_handle(tex);
                        ui.add(
                            egui::Image::new(sized)
                                .max_size(avail)
                                .maintain_aspect_ratio(true),
                        );
                    }
                });
            });
    }
}

impl eframe::App for GevstudioApp {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();

        egui::Panel::top(egui::Id::new("bar"))
            .frame(
                egui::Frame::default()
                    .fill(theme::BG_PANEL)
                    .inner_margin(egui::Margin::same(8)),
            )
            .show(root, |ui| self.top_bar(ui));

        egui::Panel::bottom(egui::Id::new("status")).show(root, |ui| {
            ui.horizontal(|ui| {
                ui.small(&self.status);
            });
        });

        egui::Panel::left(egui::Id::new("steps"))
            .resizable(true)
            .default_size(240.0)
            .show(root, |ui| self.steps_panel(ui));

        self.ensure_preview(&ctx);

        egui::CentralPanel::default().show(root, |ui| self.central(ui));
    }
}
