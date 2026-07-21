//! Réglages persistants, stockés dans `%APPDATA%\GEVSTUDIO\config.json`
//! (même dossier que la licence).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Dernier dossier utilisé pour ouvrir/enregistrer/importer.
    pub last_dir: Option<PathBuf>,
    /// Voix de narration préférée.
    pub voice: Option<String>,
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("GEVSTUDIO")
        .join("config.json")
}

impl Settings {
    pub fn load() -> Self {
        std::fs::read_to_string(config_path())
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Création du dossier de config impossible : {e}"))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Sérialisation des réglages impossible : {e}"))?;
        std::fs::write(&path, json).map_err(|e| format!("Écriture des réglages impossible : {e}"))
    }

    /// Retient le dossier parent d'un chemin comme dernier dossier utilisé.
    pub fn remember_dir(&mut self, path: &std::path::Path) {
        if let Some(dir) = path.parent() {
            self.last_dir = Some(dir.to_path_buf());
            let _ = self.save();
        }
    }
}
