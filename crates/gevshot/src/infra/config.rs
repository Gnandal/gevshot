use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Réglages persistants, stockés dans `%APPDATA%\Gevshot\config.json`.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Couleur d'annotation par défaut (RGBA).
    pub default_color: [u8; 4],
    /// Épaisseur de trait par défaut.
    pub default_stroke: f32,
    /// Délai entre le masquage de la fenêtre et la prise de vue.
    pub capture_delay_ms: u64,
    /// Dossier de sauvegarde automatique (None = Images\Gevshot).
    pub save_dir: Option<PathBuf>,
    /// Copier aussi dans le presse-papiers après « Exporter ».
    pub copy_on_save: bool,
    /// Clé API Anthropic (fonctions IA). Stockée en clair dans config.json.
    pub api_key: String,
    /// Modèle utilisé pour les fonctions IA.
    pub ai_model: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_color: [0xC6, 0x30, 0x2C, 0xFF],
            default_stroke: 3.5,
            capture_delay_ms: 400,
            save_dir: None,
            copy_on_save: false,
            api_key: String::new(),
            ai_model: "claude-opus-4-8".to_string(),
        }
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Gevshot")
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
}
