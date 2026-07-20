use eframe::egui::{self, Color32};

use crate::infra::config::Settings;
use crate::ui::{theme, widgets};

pub struct SettingsDraft {
    pub settings: Settings,
    pub section: Section,
}

impl SettingsDraft {
    pub fn new(settings: &Settings) -> Self {
        Self {
            settings: settings.clone(),
            section: Section::General,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Section {
    General,
    Capture,
    Save,
    Ai,
    Shortcuts,
}

pub enum SettingsAction {
    None,
    Cancel,
    Apply,
}

pub fn show(ui: &mut egui::Ui, draft: &mut SettingsDraft) -> SettingsAction {
    let mut action = SettingsAction::None;

    egui::Panel::left(egui::Id::new("settings_nav"))
        .exact_size(180.0)
        .show(ui, |ui| {
            ui.add_space(16.0);
            for (section, label) in [
                (Section::General, "Général"),
                (Section::Capture, "Capture"),
                (Section::Save, "Sauvegarde"),
                (Section::Ai, "IA"),
                (Section::Shortcuts, "Raccourcis"),
            ] {
                let selected = draft.section == section;
                if ui
                    .selectable_label(selected, egui::RichText::new(label).size(15.0))
                    .clicked()
                {
                    draft.section = section;
                }
                ui.add_space(4.0);
            }
        });

    egui::CentralPanel::default().show(ui, |ui| {
        ui.add_space(12.0);
        match draft.section {
            Section::General => general(ui, &mut draft.settings),
            Section::Capture => capture(ui, &mut draft.settings),
            Section::Save => save(ui, &mut draft.settings),
            Section::Ai => ai(ui, &mut draft.settings),
            Section::Shortcuts => shortcuts(ui),
        }

        let reserve = 52.0;
        ui.add_space((ui.available_height() - reserve).max(0.0));
        ui.separator();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if widgets::pill_button(ui, "Appliquer", true).clicked() {
                action = SettingsAction::Apply;
            }
            if ui.button("Annuler").clicked() {
                action = SettingsAction::Cancel;
            }
        });
    });

    action
}

fn general(ui: &mut egui::Ui, s: &mut Settings) {
    ui.heading("Général");
    ui.add_space(16.0);
    ui.label("Couleur d'annotation par défaut");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        for color in theme::PALETTE {
            if widgets::swatch(ui, color, s.default_color == color.to_array()).clicked() {
                s.default_color = color.to_array();
            }
        }
    });
    ui.add_space(16.0);
    ui.label("Épaisseur de trait par défaut");
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        for width in theme::STROKES {
            let selected = (s.default_stroke - width).abs() < 0.01;
            if widgets::stroke_dot(ui, width, selected).clicked() {
                s.default_stroke = width;
            }
        }
    });
}

fn capture(ui: &mut egui::Ui, s: &mut Settings) {
    ui.heading("Capture");
    ui.add_space(16.0);
    ui.label("Délai avant la prise de vue");
    ui.add_space(4.0);
    ui.add(
        egui::Slider::new(&mut s.capture_delay_ms, 200..=1500)
            .step_by(50.0)
            .suffix(" ms"),
    );
    ui.add_space(4.0);
    ui.weak("Le temps que la fenêtre se masque avant que l'écran soit figé.");
}

fn save(ui: &mut egui::Ui, s: &mut Settings) {
    ui.heading("Sauvegarde");
    ui.add_space(16.0);
    ui.label("Dossier de sauvegarde automatique");
    ui.add_space(4.0);
    let current = s
        .save_dir
        .clone()
        .unwrap_or_else(crate::infra::export::default_save_dir);
    ui.horizontal(|ui| {
        egui::Frame::default()
            .fill(Color32::WHITE)
            .stroke(egui::Stroke::new(1.0, theme::BORDER))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.monospace(current.display().to_string());
            });
        if ui.button("Parcourir…").clicked()
            && let Some(dir) = rfd::FileDialog::new().set_directory(&current).pick_folder()
        {
            s.save_dir = Some(dir);
        }
    });
    if s.save_dir.is_some() {
        ui.add_space(4.0);
        if ui
            .small_button("Revenir au dossier par défaut (Images\\Gevshot)")
            .clicked()
        {
            s.save_dir = None;
        }
    }
    ui.add_space(16.0);
    ui.checkbox(
        &mut s.copy_on_save,
        "Copier aussi dans le presse-papiers après « Exporter »",
    );
}

fn ai(ui: &mut egui::Ui, s: &mut Settings) {
    ui.heading("IA");
    ui.add_space(8.0);
    ui.weak(
        "Génération de légende et annotation sur instruction, via l'API Claude \
         (Anthropic). Nécessite une clé API : console.anthropic.com → API Keys.",
    );
    ui.add_space(16.0);
    ui.label("Clé API Anthropic");
    ui.add_space(4.0);
    ui.add(
        egui::TextEdit::singleline(&mut s.api_key)
            .password(true)
            .desired_width(320.0)
            .hint_text("sk-ant-…"),
    );
    ui.add_space(4.0);
    ui.weak("Stockée en clair dans le fichier config.json de l'application.");
    ui.add_space(16.0);
    ui.label("Modèle");
    ui.add_space(4.0);
    let models = [
        ("claude-opus-4-8", "Opus 4.8 — qualité maximale"),
        ("claude-sonnet-5", "Sonnet 5 — équilibré"),
        ("claude-haiku-4-5", "Haiku 4.5 — rapide et économique"),
    ];
    let current = models
        .iter()
        .find(|(id, _)| *id == s.ai_model)
        .map(|(_, label)| *label)
        .unwrap_or(s.ai_model.as_str());
    egui::ComboBox::from_id_salt("ai_model")
        .selected_text(current)
        .width(280.0)
        .show_ui(ui, |ui| {
            for (id, label) in models {
                ui.selectable_value(&mut s.ai_model, id.to_string(), label);
            }
        });
    ui.add_space(4.0);
    ui.weak("Coût indicatif : quelques centimes par capture, selon le modèle.");
}

fn shortcuts(ui: &mut egui::Ui) {
    ui.heading("Raccourcis");
    ui.add_space(16.0);
    egui::Grid::new("shortcuts_grid")
        .num_columns(2)
        .spacing([32.0, 10.0])
        .show(ui, |ui| {
            for (action, keys) in [
                ("Capturer (global)", "Ctrl+Shift+S"),
                ("Copier l'image annotée", "Ctrl+C"),
                ("Exporter en PNG", "Ctrl+S"),
                ("Annuler / Rétablir", "Ctrl+Z / Ctrl+Y"),
                ("Supprimer la sélection", "Suppr"),
                ("Tout l'écran (pendant la sélection)", "Entrée"),
                ("Annuler la capture", "Échap"),
            ] {
                ui.label(action);
                ui.label(
                    egui::RichText::new(keys)
                        .monospace()
                        .color(theme::TEXT_WEAK),
                );
                ui.end_row();
            }
        });
}
