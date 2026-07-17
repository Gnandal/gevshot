//! Polices du rendu d'export (aplatissement sur pixels, PDF).
//!
//! Deux sources selon la feature : `system-fonts` lit C:\Windows\Fonts
//! (comportement historique de GEVSHOT — un emplacement introuvable est
//! simplement sauté au rendu), `embedded-fonts` embarque les fontes DejaVu
//! (sans/serif/mono × normal/gras) pour un rendu déterministe
//! multi-plateforme (GEVSTUDIO). Voir `fonts/LICENSE`.

use ab_glyph::FontVec;

use crate::model::FontChoice;

/// Familles proposées, dans l'ordre `(font, bold)` → index = font*2 + bold.
/// Chaque entrée liste les fichiers candidats dans C:\Windows\Fonts.
const CANDIDATES: [(&str, &[&str]); 6] = [
    ("sans", &["segoeui.ttf", "arial.ttf"]),
    ("sans-b", &["segoeuib.ttf", "arialbd.ttf"]),
    ("serif", &["georgia.ttf", "times.ttf"]),
    ("serif-b", &["georgiab.ttf", "timesbd.ttf"]),
    ("mono", &["consola.ttf", "cour.ttf"]),
    ("mono-b", &["consolab.ttf", "courbd.ttf"]),
];

/// Nom stable de la famille (clé egui côté éditeur).
pub fn family_name(font: FontChoice, bold: bool) -> &'static str {
    CANDIDATES[index(font, bold)].0
}

fn index(font: FontChoice, bold: bool) -> usize {
    let f = match font {
        FontChoice::Sans => 0,
        FontChoice::Serif => 1,
        FontChoice::Mono => 2,
    };
    f * 2 + usize::from(bold)
}

#[cfg(feature = "system-fonts")]
fn read_first(files: &[&str]) -> Option<Vec<u8>> {
    let dir = std::path::Path::new("C:\\Windows\\Fonts");
    files.iter().find_map(|f| std::fs::read(dir.join(f)).ok())
}

/// Octets TTF de la police système pour une famille (glue egui côté app).
#[cfg(feature = "system-fonts")]
pub fn system_font_bytes(font: FontChoice, bold: bool) -> Option<Vec<u8>> {
    read_first(CANDIDATES[index(font, bold)].1)
}

/// Jeu de six polices (sans/serif/mono × normal/gras) pour le rendu pixels.
/// Un emplacement absent est sauté au rendu (comportement historique).
/// Les octets TTF sont conservés pour l'embarquement PDF.
pub struct FontSet {
    slots: Vec<Option<(Vec<u8>, FontVec)>>,
}

impl FontSet {
    /// Polices système Windows (comportement historique de GEVSHOT).
    #[cfg(feature = "system-fonts")]
    pub fn system() -> FontSet {
        let slots = CANDIDATES
            .iter()
            .map(|(_, files)| read_first(files).and_then(slot_from_bytes))
            .collect();
        FontSet { slots }
    }

    /// Fontes DejaVu embarquées : rendu identique sur toutes les
    /// plateformes, indépendant des polices installées (GEVSTUDIO, exports).
    #[cfg(feature = "embedded-fonts")]
    pub fn embedded() -> FontSet {
        const EMBEDDED: [&[u8]; 6] = [
            include_bytes!("../fonts/DejaVuSans.ttf"),
            include_bytes!("../fonts/DejaVuSans-Bold.ttf"),
            include_bytes!("../fonts/DejaVuSerif.ttf"),
            include_bytes!("../fonts/DejaVuSerif-Bold.ttf"),
            include_bytes!("../fonts/DejaVuSansMono.ttf"),
            include_bytes!("../fonts/DejaVuSansMono-Bold.ttf"),
        ];
        FontSet {
            slots: EMBEDDED
                .iter()
                .map(|b| slot_from_bytes(b.to_vec()))
                .collect(),
        }
    }

    /// Jeu vide : aucune police, tout texte est sauté au rendu (tests).
    pub fn empty() -> FontSet {
        FontSet {
            slots: (0..CANDIDATES.len()).map(|_| None).collect(),
        }
    }

    /// Construit un jeu à partir d'octets TTF fournis, dans l'ordre
    /// sans, sans-gras, serif, serif-gras, mono, mono-gras.
    pub fn from_bytes(bytes: [Option<Vec<u8>>; 6]) -> FontSet {
        FontSet {
            slots: bytes
                .into_iter()
                .map(|b| b.and_then(slot_from_bytes))
                .collect(),
        }
    }

    pub fn get(&self, font: FontChoice, bold: bool) -> Option<&FontVec> {
        self.slots[index(font, bold)].as_ref().map(|(_, f)| f)
    }

    /// Octets TTF de la police (embarquement dans un PDF).
    pub fn bytes(&self, font: FontChoice, bold: bool) -> Option<&[u8]> {
        self.slots[index(font, bold)]
            .as_ref()
            .map(|(b, _)| b.as_slice())
    }
}

fn slot_from_bytes(bytes: Vec<u8>) -> Option<(Vec<u8>, FontVec)> {
    FontVec::try_from_vec(bytes.clone()).ok().map(|f| (bytes, f))
}
