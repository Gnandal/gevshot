//! Sorties fichier et presse-papiers : image aplatie, PNG auto-nommé,
//! « Enregistrer sous », projet .gev.

use eframe::egui;

use super::{Capture, GevshotApp, Stage};
use crate::infra::{export, project};

impl GevshotApp {
    /// Image finale : capture + annotations aplaties.
    fn rendered_image(&self) -> Option<image::RgbaImage> {
        let Stage::Editing { capture } = &self.stage else {
            return None;
        };
        Some(export::flatten(&capture.image, self.editor.annotations()))
    }

    pub(super) fn copy_to_clipboard(&mut self) {
        let Some(img) = self.rendered_image() else {
            return;
        };
        match export::copy_to_clipboard(&img) {
            Ok(()) => self.set_status("Copié dans le presse-papiers"),
            Err(e) => self.set_error(e),
        }
    }

    pub(super) fn save_auto(&mut self, ctx: &egui::Context) {
        let Some(img) = self.rendered_image() else {
            return;
        };
        match export::save_auto(&img, &self.save_dir()) {
            Ok(path) => {
                if let Some(stem) = path.file_stem() {
                    self.doc_name = stem.to_string_lossy().into_owned();
                    self.update_title(ctx);
                }
                if self.settings.copy_on_save {
                    match export::copy_to_clipboard(&img) {
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
    pub(super) fn save_dialog(&mut self) {
        let Some(img) = self.rendered_image() else {
            return;
        };
        match export::save_dialog(&img, &self.save_dir()) {
            Ok(Some(path)) => self.set_status(format!("Sauvé : {}", path.display())),
            Ok(None) => {}
            Err(e) => self.set_error(e),
        }
    }

    pub(super) fn save_project(&mut self) {
        let Stage::Editing { capture } = &self.stage else {
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Projet GEVSHOT", &[project::EXTENSION])
            .set_file_name(format!(
                "{}.{}",
                if self.doc_name.is_empty() {
                    "capture"
                } else {
                    &self.doc_name
                },
                project::EXTENSION
            ))
            .set_directory(self.save_dir())
            .save_file()
        else {
            return;
        };
        match project::save(&path, &capture.image, self.editor.annotations()) {
            Ok(()) => self.set_status(format!("Projet sauvé : {}", path.display())),
            Err(e) => self.set_error(e),
        }
    }

    pub(super) fn open_project(&mut self, ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Projet GEVSHOT", &[project::EXTENSION])
            .set_directory(self.save_dir())
            .pick_file()
        else {
            return;
        };
        match project::load(&path) {
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
}
