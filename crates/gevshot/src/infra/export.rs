//! Sorties de l'image annotée : presse-papiers, PNG auto-nommé, dialogues.
//! L'aplatissement lui-même vit dans `gev_core::render`.

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use crate::domain::annotation::Annotation;

/// Rend les annotations sur une copie de l'image, avec les polices système.
pub fn flatten(image: &image::RgbaImage, annotations: &[Annotation]) -> image::RgbaImage {
    gev_core::render::flatten(image, annotations, super::fonts::export_fonts())
}

/// Copie du texte dans le presse-papiers.
pub fn copy_text(text: &str) -> Result<(), String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("Presse-papiers indisponible : {e}"))?;
    clipboard
        .set_text(text.to_string())
        .map_err(|e| format!("Copie impossible : {e}"))
}

/// Copie l'image dans le presse-papiers.
pub fn copy_to_clipboard(img: &image::RgbaImage) -> Result<(), String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("Presse-papiers indisponible : {e}"))?;
    clipboard
        .set_image(arboard::ImageData {
            width: img.width() as usize,
            height: img.height() as usize,
            bytes: Cow::Borrowed(img.as_raw()),
        })
        .map_err(|e| format!("Copie impossible : {e}"))
}

/// Dossier de sauvegarde automatique par défaut : Images\Gevshot.
pub fn default_save_dir() -> PathBuf {
    dirs::picture_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Gevshot")
}

/// Prochain nom auto-incrémenté (sans extension) : `capture-AAAA-MM-JJ-NNN`.
pub fn next_auto_name(dir: &Path) -> String {
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let prefix = format!("capture-{date}-");
    let next = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            name.strip_prefix(&prefix)?
                .strip_suffix(".png")?
                .parse::<u32>()
                .ok()
        })
        .max()
        .unwrap_or(0)
        + 1;
    format!("{prefix}{next:03}")
}

/// Sauvegarde en PNG sous un nom auto-incrémenté dans `dir`.
pub fn save_auto(img: &image::RgbaImage, dir: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("Création du dossier impossible : {e}"))?;
    let path = dir.join(format!("{}.png", next_auto_name(dir)));
    img.save(&path)
        .map_err(|e| format!("Écriture impossible : {e}"))?;
    Ok(path)
}

/// Sauvegarde via une boîte de dialogue « Enregistrer sous ».
pub fn save_dialog(img: &image::RgbaImage, dir: &Path) -> Result<Option<PathBuf>, String> {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("Image PNG", &["png"])
        .set_file_name(format!("{}.png", next_auto_name(dir)))
        .set_directory(dir)
        .save_file()
    else {
        return Ok(None);
    };
    img.save(&path)
        .map_err(|e| format!("Écriture impossible : {e}"))?;
    Ok(Some(path))
}
