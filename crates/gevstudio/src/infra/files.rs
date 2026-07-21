//! Dialogues de fichiers : ouverture/enregistrement du document `.gevd` et
//! import de captures (PNG/JPEG ou projets `.gev` de GEVSHOT).

use std::path::{Path, PathBuf};

use gev_core::document::{self, Document, Step, Transition};
use gev_core::project;

/// Étape neuve à partir d'une image et d'annotations éventuelles.
fn step_from(image: image::RgbaImage, annotations: Vec<gev_core::model::Annotation>, title: String) -> Step {
    Step {
        image,
        annotations,
        title,
        caption: String::new(),
        narration: String::new(),
        narration_audio: None,
        duration_override_ms: None,
        transition: Transition::Cut,
    }
}

/// Ouvre le sélecteur d'import (multi-sélection) et charge chaque fichier en
/// étape. Renvoie les étapes chargées et les erreurs rencontrées.
pub fn import_steps(last_dir: Option<&Path>) -> (Vec<Step>, Vec<String>) {
    let mut dialog = rfd::FileDialog::new()
        .add_filter("Captures (PNG, JPEG, GEV)", &["png", "jpg", "jpeg", "gev"])
        .set_title("Importer des captures");
    if let Some(dir) = last_dir {
        dialog = dialog.set_directory(dir);
    }
    let Some(paths) = dialog.pick_files() else {
        return (Vec::new(), Vec::new());
    };
    load_steps(&paths)
}

/// Charge une liste de fichiers (import ou glisser-déposer) en étapes.
pub fn load_steps(paths: &[PathBuf]) -> (Vec<Step>, Vec<String>) {
    let mut steps = Vec::new();
    let mut errors = Vec::new();
    for path in paths {
        let title = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .unwrap_or_default();
        let result = if ext == project::EXTENSION {
            project::load(path).map(|(image, annotations)| step_from(image, annotations, title.clone()))
        } else {
            image::open(path)
                .map(|img| step_from(img.to_rgba8(), Vec::new(), title.clone()))
                .map_err(|e| format!("Image illisible : {e}"))
        };
        match result {
            Ok(step) => steps.push(step),
            Err(e) => errors.push(format!("{} : {e}", path.display())),
        }
    }
    (steps, errors)
}

/// Extensions acceptées par l'import (pour filtrer le glisser-déposer).
pub fn importable(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .as_deref(),
        Some("png") | Some("jpg") | Some("jpeg") | Some("gev")
    )
}

/// Dialogue « Ouvrir un guide ».
pub fn open_dialog(last_dir: Option<&Path>) -> Option<(PathBuf, Result<Document, String>)> {
    let mut dialog = rfd::FileDialog::new()
        .add_filter("Guide GEVSTUDIO", &[document::EXTENSION])
        .set_title("Ouvrir un guide");
    if let Some(dir) = last_dir {
        dialog = dialog.set_directory(dir);
    }
    let path = dialog.pick_file()?;
    let result = document::load(&path);
    Some((path, result))
}

/// Dialogue « Enregistrer sous » ; renvoie le chemin choisi (extension forcée).
pub fn save_dialog(doc_title: &str, last_dir: Option<&Path>) -> Option<PathBuf> {
    let name = if doc_title.trim().is_empty() {
        "guide".to_string()
    } else {
        doc_title.trim().replace(['/', '\\', ':'], "-")
    };
    let mut dialog = rfd::FileDialog::new()
        .add_filter("Guide GEVSTUDIO", &[document::EXTENSION])
        .set_file_name(format!("{name}.{}", document::EXTENSION))
        .set_title("Enregistrer le guide");
    if let Some(dir) = last_dir {
        dialog = dialog.set_directory(dir);
    }
    let mut path = dialog.save_file()?;
    if path
        .extension()
        .and_then(|e| e.to_str())
        .is_none_or(|e| !e.eq_ignore_ascii_case(document::EXTENSION))
    {
        path.set_extension(document::EXTENSION);
    }
    Some(path)
}
