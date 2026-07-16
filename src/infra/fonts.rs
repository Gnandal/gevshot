use std::path::Path;
use std::sync::OnceLock;

use ab_glyph::FontVec;
use eframe::egui;

use crate::domain::annotation::FontChoice;

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

fn read_first(files: &[&str]) -> Option<Vec<u8>> {
    let dir = Path::new("C:\\Windows\\Fonts");
    files.iter().find_map(|f| std::fs::read(dir.join(f)).ok())
}

/// Enregistre les familles système dans egui (éditeur).
/// Les familles introuvables retombent sur la police proportionnelle par défaut.
pub fn install(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let default_family = fonts
        .families
        .get(&egui::FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    for (name, files) in CANDIDATES {
        let family = egui::FontFamily::Name(name.into());
        match read_first(files) {
            Some(bytes) => {
                fonts.font_data.insert(
                    name.to_string(),
                    std::sync::Arc::new(egui::FontData::from_owned(bytes)),
                );
                fonts.families.insert(family, vec![name.to_string()]);
            }
            None => {
                fonts.families.insert(family, default_family.clone());
            }
        }
    }
    ctx.set_fonts(fonts);
}

/// Même police, côté export (rendu sur pixels via ab_glyph).
pub fn export_font(font: FontChoice, bold: bool) -> Option<&'static FontVec> {
    static CACHE: OnceLock<Vec<Option<FontVec>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| {
        CANDIDATES
            .iter()
            .map(|(_, files)| read_first(files).and_then(|b| FontVec::try_from_vec(b).ok()))
            .collect()
    });
    cache[index(font, bold)].as_ref()
}
