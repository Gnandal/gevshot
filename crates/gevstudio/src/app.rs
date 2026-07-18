//! Interface GEVSTUDIO (Direction B) : barre d'outils, timeline des étapes avec
//! vignettes, aperçu, édition titre/légende, export PDF/PPTX/GIF, et écran de
//! monétisation (déblocage Pro par licence).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use eframe::egui::{self, FontFamily, FontId};
use gev_core::document::{self, Document, Step};
use gev_core::fonts::FontSet;
use gev_core::{project, render};

use crate::license::License;
use crate::{gif, pdf, pptx, theme};

const BUY_URL: &str = "https://gevstudio.app/pro";
const PRICE: &str = "39 €";

#[derive(Clone, Copy, PartialEq)]
enum ExportFormat {
    Pdf,
    Pptx,
    Gif,
}

impl ExportFormat {
    fn ext(self) -> &'static str {
        match self {
            ExportFormat::Pdf => "pdf",
            ExportFormat::Pptx => "pptx",
            ExportFormat::Gif => "gif",
        }
    }
    fn label(self) -> &'static str {
        match self {
            ExportFormat::Pdf => "PDF",
            ExportFormat::Pptx => "PPTX",
            ExportFormat::Gif => "GIF animé",
        }
    }
    /// Vrai si le format est réservé à la version Pro.
    fn is_pro_only(self) -> bool {
        !matches!(self, ExportFormat::Pdf)
    }
}

/// Lance l'interface. Un `.gevd` optionnel est ouvert au démarrage.
pub fn launch(open: Option<PathBuf>) -> ExitCode {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1160.0, 780.0])
            .with_min_inner_size([820.0, 560.0])
            .with_title("GEVSTUDIO"),
        ..Default::default()
    };
    let result = eframe::run_native(
        "GEVSTUDIO",
        options,
        Box::new(move |cc| {
            theme::install_fonts(&cc.egui_ctx);
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
    preview: Option<egui::TextureHandle>,
    preview_for: Option<usize>,
    thumbs: Vec<egui::TextureHandle>,
    thumbs_dirty: bool,
    status: String,
    license: License,
    // Écran « Passer à Pro ».
    show_pro: bool,
    key_input: String,
    key_msg: Option<Result<String, String>>,
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
            thumbs: Vec::new(),
            thumbs_dirty: true,
            status: "Nouveau document.".into(),
            license: License::load(),
            show_pro: false,
            key_input: String::new(),
            key_msg: None,
        };
        if let Some(p) = open {
            app.open_path(&p);
        }
        app
    }

    fn is_pro(&self) -> bool {
        self.license.is_pro()
    }

    // ----- Fichiers ----------------------------------------------------------

    fn open_path(&mut self, path: &Path) {
        match document::load(path) {
            Ok(doc) => {
                let n = doc.steps.len();
                self.doc = doc;
                self.path = Some(path.to_path_buf());
                self.selected = 0;
                self.invalidate();
                self.status = format!("Ouvert · {n} étape(s)");
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
                self.path = Some(path);
                self.status = "Enregistré".into();
            }
            Err(e) => self.status = format!("Enregistrement impossible : {e}"),
        }
    }

    fn export(&mut self, format: ExportFormat) {
        if format.is_pro_only() && !self.is_pro() {
            self.show_pro = true;
            return;
        }
        if self.doc.steps.is_empty() {
            self.status = "Rien à exporter : le document est vide.".into();
            return;
        }
        let ext = format.ext();
        let default_name = self
            .path
            .as_ref()
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .map(|s| format!("{s}.{ext}"))
            .unwrap_or_else(|| format!("guide.{ext}"));
        let Some(path) = rfd::FileDialog::new()
            .add_filter(format.label(), &[ext])
            .set_file_name(&default_name)
            .save_file()
        else {
            return;
        };
        let built = match format {
            ExportFormat::Pdf => pdf::build(&self.doc, !self.is_pro()),
            ExportFormat::Pptx => pptx::build(&self.doc),
            ExportFormat::Gif => gif::build(&self.doc),
        };
        match built.and_then(|bytes| {
            std::fs::write(&path, &bytes).map_err(|e| format!("écriture impossible : {e}"))
        }) {
            Ok(()) => self.status = format!("{} exporté", format.label()),
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
                self.status = "Étape ajoutée".into();
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
                self.status = "Étape ajoutée".into();
            }
            Err(e) => self.status = format!("Capture illisible : {e}"),
        }
    }

    fn push_step(&mut self, step: Step) {
        self.doc.steps.push(step);
        self.selected = self.doc.steps.len() - 1;
        self.invalidate();
    }

    // ----- Timeline ----------------------------------------------------------

    fn select(&mut self, i: usize) {
        if i != self.selected {
            self.selected = i;
            self.preview = None;
            self.preview_for = None;
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
        self.invalidate();
    }

    fn delete_step(&mut self, i: usize) {
        if i >= self.doc.steps.len() {
            return;
        }
        self.doc.steps.remove(i);
        self.selected = self.selected.min(self.doc.steps.len().saturating_sub(1));
        self.invalidate();
    }

    fn invalidate(&mut self) {
        self.preview = None;
        self.preview_for = None;
        self.thumbs_dirty = true;
    }

    // ----- Textures ----------------------------------------------------------

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
        self.preview = Some(ctx.load_texture("preview", color, egui::TextureOptions::LINEAR));
        self.preview_for = Some(self.selected);
    }

    fn ensure_thumbs(&mut self, ctx: &egui::Context) {
        if !self.thumbs_dirty {
            return;
        }
        self.thumbs.clear();
        for (i, step) in self.doc.steps.iter().enumerate() {
            // Vignette : capture brute réduite (rapide, sans réaplatir).
            let small = image::imageops::resize(
                &step.image,
                120,
                (120.0 * step.image.height() as f32 / step.image.width().max(1) as f32) as u32,
                image::imageops::FilterType::Triangle,
            );
            let size = [small.width() as usize, small.height() as usize];
            let color = egui::ColorImage::from_rgba_unmultiplied(size, small.as_raw());
            self.thumbs
                .push(ctx.load_texture(format!("thumb{i}"), color, egui::TextureOptions::LINEAR));
        }
        self.thumbs_dirty = false;
    }

    // ----- Rendu -------------------------------------------------------------

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(theme::semibold("GEVSTUDIO").size(15.0));
            ui.add_space(2.0);
            ui.add_space(8.0);
            if ui.button("Ouvrir").clicked() {
                self.open_dialog();
            }
            if ui.button("Enregistrer").clicked() {
                self.save(false);
            }
            ui.separator();
            if ui.button("Ajouter une image").clicked() {
                self.add_step_from_image();
            }
            if ui.button("Depuis .gev").clicked() {
                self.add_step_from_gev();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if theme::primary_button(ui, "Exporter PDF").clicked() {
                    self.export(ExportFormat::Pdf);
                }
                if self.export_button(ui, ExportFormat::Pptx).clicked() {
                    self.export(ExportFormat::Pptx);
                }
                if self.export_button(ui, ExportFormat::Gif).clicked() {
                    self.export(ExportFormat::Gif);
                }
            });
        });
    }

    /// Bouton d'export secondaire, avec mention « Pro » si verrouillé.
    fn export_button(&self, ui: &mut egui::Ui, format: ExportFormat) -> egui::Response {
        let locked = format.is_pro_only() && !self.is_pro();
        let label = if locked {
            format!("Exporter {}  ·  Pro", format.label())
        } else {
            format!("Exporter {}", format.label())
        };
        let mut text = theme::medium(label);
        if locked {
            text = text.color(theme::MUTED);
        }
        ui.add(egui::Button::new(text))
    }

    fn steps_panel(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(theme::meta("Étapes"));
            ui.label(theme::semibold(self.doc.steps.len().to_string()).size(12.0));
        });
        ui.add_space(6.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            let count = self.doc.steps.len();
            for i in 0..count {
                let title = self.doc.steps[i].title.clone();
                let thumb = self.thumbs.get(i).cloned();
                self.step_row(ui, i, &title, thumb.as_ref());
            }
            ui.add_space(6.0);
            if ui
                .add(egui::Button::new(theme::medium("+  Ajouter une étape").color(theme::MUTED)).frame(false))
                .clicked()
            {
                self.add_step_from_image();
            }
        });
    }

    fn step_row(
        &mut self,
        ui: &mut egui::Ui,
        i: usize,
        title: &str,
        thumb: Option<&egui::TextureHandle>,
    ) {
        let selected = i == self.selected;
        // Direction B : l'étape sélectionnée porte une fine bordure accent
        // (seul emploi du bleu) ; les autres n'ont pas de bordure.
        let mut frame = egui::Frame::default()
            .corner_radius(egui::CornerRadius::same(10))
            .inner_margin(egui::Margin::same(8));
        if selected {
            frame = frame
                .fill(theme::WHITE)
                .stroke(egui::Stroke::new(1.0, theme::ACCENT));
        }

        let resp = frame
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Numéro d'étape : accent quand sélectionné.
                    let num = if selected {
                        theme::medium((i + 1).to_string())
                            .size(11.0)
                            .color(theme::ACCENT)
                    } else {
                        theme::meta(&(i + 1).to_string())
                    };
                    ui.label(num);
                    // Vignette.
                    if let Some(tex) = thumb {
                        let sized = egui::load::SizedTexture::from_handle(tex);
                        ui.add(
                            egui::Image::new(sized)
                                .fit_to_exact_size(egui::vec2(56.0, 36.0))
                                .corner_radius(6.0),
                        );
                    } else {
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(56.0, 36.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 6.0, theme::SUBTLE);
                    }
                    ui.add_space(4.0);
                    if title.is_empty() {
                        ui.label(theme::medium("Sans titre").color(theme::MUTED));
                    } else {
                        ui.label(theme::medium(title));
                    }
                });
            })
            .response;

        if resp.interact(egui::Sense::click()).clicked() {
            self.select(i);
        }

        if selected {
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                if ui.small_button("Monter").clicked() {
                    self.move_step(i, -1);
                }
                if ui.small_button("Descendre").clicked() {
                    self.move_step(i, 1);
                }
                if theme::danger_button(ui, "Supprimer").clicked() {
                    self.delete_step(i);
                }
            });
        }
        ui.add_space(6.0);
    }

    fn central(&mut self, ui: &mut egui::Ui) {
        if self.doc.steps.is_empty() {
            self.empty_state(ui);
            return;
        }
        let selected = self.selected;
        let n = self.doc.steps.len();

        // Fil d'ariane.
        let guide = if self.doc.title.is_empty() {
            "Sans titre".to_string()
        } else {
            self.doc.title.clone()
        };
        ui.add_space(6.0);
        ui.label(theme::meta(&format!("{guide} · Étape {} sur {n}", selected + 1)));
        ui.add_space(4.0);

        // Titre de l'étape (grand, éditable).
        if let Some(step) = self.doc.steps.get_mut(selected) {
            ui.add(
                egui::TextEdit::singleline(&mut step.title)
                    .font(FontId::new(22.0, FontFamily::Name("inter-semibold".into())))
                    .frame(egui::Frame::default())
                    .hint_text("Titre de l'étape")
                    .desired_width(f32::INFINITY),
            );
        }
        ui.add_space(8.0);

        // Légende ancrée en bas ; l'aperçu occupe le reste.
        egui::Panel::bottom(egui::Id::new("caption"))
            .frame(egui::Frame::default().inner_margin(egui::Margin::symmetric(0, 8)))
            .show(ui, |ui| {
                ui.label(theme::meta("Légende"));
                ui.add_space(4.0);
                if let Some(step) = self.doc.steps.get_mut(selected) {
                    ui.add(
                        egui::TextEdit::multiline(&mut step.caption)
                            .hint_text("Texte affiché sous la capture")
                            .desired_width(f32::INFINITY)
                            .desired_rows(3),
                    );
                }
            });

        // Aperçu.
        egui::Frame::default()
            .fill(theme::CANVAS)
            .stroke(egui::Stroke::new(1.0, theme::HAIRLINE))
            .corner_radius(egui::CornerRadius::same(12))
            .inner_margin(egui::Margin::same(14))
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

    fn empty_state(&mut self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(40.0);
                ui.label(theme::semibold("Créez votre premier guide").size(20.0));
                ui.add_space(6.0);
                ui.label(
                    theme::medium("Importez une capture, ou ouvrez un document existant.")
                        .color(theme::MUTED),
                );
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() / 2.0 - 130.0);
                    if theme::primary_button(ui, "Ajouter une image").clicked() {
                        self.add_step_from_image();
                    }
                    if ui.button("Ouvrir").clicked() {
                        self.open_dialog();
                    }
                });
            });
        });
    }

    fn status_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.colored_label(theme::SUCCESS, "●");
            ui.label(theme::medium(&self.status).size(12.0).color(theme::MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(name) = self.license.licensee() {
                    ui.label(
                        theme::semibold(format!("GEVSTUDIO Pro · {name}"))
                            .size(12.0)
                            .color(theme::INK),
                    );
                } else if !crate::license::FREE_MODE {
                    // Upsell seulement quand la vente est active.
                    let tier = theme::medium("Version gratuite").size(12.0).color(theme::MUTED);
                    if ui
                        .add(egui::Label::new(tier).sense(egui::Sense::click()))
                        .clicked()
                    {
                        self.show_pro = true;
                    }
                }
            });
        });
    }

    // ----- Écran Pro ---------------------------------------------------------

    fn pro_window(&mut self, ctx: &egui::Context) {
        if !self.show_pro {
            return;
        }
        let mut open = true;
        egui::Window::new(theme::semibold("Passez à GEVSTUDIO Pro").size(18.0))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .open(&mut open)
            .show(ctx, |ui| {
                ui.set_width(460.0);
                ui.add_space(4.0);
                ui.label(
                    theme::medium(
                        "Export sans filigrane, PowerPoint et vidéo/GIF. Paiement unique, aucune limite dans le temps.",
                    )
                    .color(theme::MUTED),
                );
                ui.add_space(12.0);

                egui::Grid::new("free_pro")
                    .num_columns(2)
                    .spacing(egui::vec2(40.0, 6.0))
                    .show(ui, |ui| {
                        ui.label(theme::meta("Gratuit"));
                        ui.label(theme::meta("Pro"));
                        ui.end_row();
                        ui.label("Guides illimités");
                        ui.label("Tout le gratuit");
                        ui.end_row();
                        ui.label("PDF (filigrane)");
                        ui.label(theme::medium("PDF sans filigrane"));
                        ui.end_row();
                        ui.label(theme::medium("PowerPoint").color(theme::MUTED));
                        ui.label(theme::medium("PowerPoint"));
                        ui.end_row();
                        ui.label(theme::medium("Vidéo / GIF").color(theme::MUTED));
                        ui.label(theme::medium("Vidéo / GIF"));
                        ui.end_row();
                    });

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.label(theme::semibold(PRICE).size(22.0));
                    ui.label(theme::medium("· paiement unique, à vie").color(theme::MUTED));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::primary_button(ui, "Débloquer Pro").clicked() {
                            ctx.open_url(egui::OpenUrl::new_tab(BUY_URL));
                        }
                    });
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(8.0);
                ui.label(theme::semibold("Activer votre licence").size(14.0));
                ui.label(theme::medium("Collez la clé reçue par e-mail.").color(theme::MUTED));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.key_input)
                            .hint_text("Collez votre licence…")
                            .desired_width(260.0),
                    );
                    if theme::primary_button(ui, "Activer").clicked() {
                        match self.license.activate(&self.key_input) {
                            Ok(()) => {
                                self.key_msg =
                                    Some(Ok("Licence activée — Pro débloqué.".into()));
                                self.status = "GEVSTUDIO Pro activé".into();
                            }
                            Err(e) => self.key_msg = Some(Err(e)),
                        }
                    }
                });
                if let Some(msg) = &self.key_msg {
                    ui.add_space(4.0);
                    match msg {
                        Ok(t) => ui.colored_label(theme::SUCCESS, t),
                        Err(e) => ui.colored_label(theme::DANGER, e),
                    };
                }
            });
        // Fermeture (croix ou activation réussie).
        if !open || self.is_pro() {
            self.show_pro = false;
        }
    }
}

impl eframe::App for GevstudioApp {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = root.ctx().clone();

        egui::Panel::top(egui::Id::new("bar"))
            .frame(
                egui::Frame::default()
                    .fill(theme::WHITE)
                    .stroke(egui::Stroke::new(1.0, theme::HAIRLINE))
                    .inner_margin(egui::Margin::symmetric(12, 10)),
            )
            .show(root, |ui| self.top_bar(ui));

        egui::Panel::bottom(egui::Id::new("status"))
            .frame(
                egui::Frame::default()
                    .fill(theme::SUBTLE)
                    .inner_margin(egui::Margin::symmetric(12, 6)),
            )
            .show(root, |ui| self.status_bar(ui));

        egui::Panel::left(egui::Id::new("steps"))
            .resizable(true)
            .default_size(268.0)
            .frame(
                egui::Frame::default()
                    .fill(theme::WHITE)
                    .stroke(egui::Stroke::new(1.0, theme::HAIRLINE))
                    .inner_margin(egui::Margin::symmetric(12, 4)),
            )
            .show(root, |ui| self.steps_panel(ui));

        self.ensure_thumbs(&ctx);
        self.ensure_preview(&ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(theme::WHITE).inner_margin(egui::Margin::same(20)))
            .show(root, |ui| self.central(ui));

        self.pro_window(&ctx);
    }
}
