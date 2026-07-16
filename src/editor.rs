use eframe::egui::{self, Color32, CursorIcon, Pos2, Rect, Stroke, Vec2};

use crate::annotation::{Annotation, CanvasTransform, FontChoice, Shape};
use crate::{theme, widgets};

/// Tolérance de sélection en pixels écran.
const HIT_TOLERANCE: f32 = 6.0;

/// Fonction de dessin d'une icône d'outil.
type ToolIcon = fn(&egui::Painter, Pos2, Color32);

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Select,
    Badge,
    Arrow,
    Rect,
    Ellipse,
    Text,
}

/// Session de saisie de texte en cours sur le canevas.
struct TextEditState {
    index: usize,
    buffer: String,
    /// Texte d'origine (édition d'une annotation existante).
    original: Option<String>,
    just_created: bool,
    request_focus: bool,
}

/// Édition du numéro d'une pastille (double-clic).
struct BadgeEditState {
    index: usize,
    buffer: String,
    request_focus: bool,
}

enum Drag {
    /// Tracé d'une nouvelle forme (coordonnées image).
    Draw { start: Pos2, current: Pos2 },
    /// Déplacement de l'annotation sélectionnée ; `before` sert au undo.
    Move {
        last: Pos2,
        before: Vec<Annotation>,
        moved: bool,
    },
}

pub struct Editor {
    annotations: Vec<Annotation>,
    undo_stack: Vec<Vec<Annotation>>,
    redo_stack: Vec<Vec<Annotation>>,
    tool: Tool,
    color: Color32,
    stroke_width: f32,
    selected: Option<usize>,
    drag: Option<Drag>,
    /// Dernière échelle d'affichage (pour la barre d'état).
    last_scale: f32,
    editing_text: Option<TextEditState>,
    /// Style courant de l'outil texte.
    text_size: f32,
    text_font: FontChoice,
    text_bold: bool,
    editing_badge: Option<BadgeEditState>,
    /// Premier numéro de la séquence de pastilles.
    badge_start: u32,
    /// Anneau blanc autour des prochaines pastilles.
    badge_border: bool,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            annotations: Vec::new(),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            tool: Tool::Select,
            color: theme::PALETTE[0],
            stroke_width: theme::STROKES[1],
            selected: None,
            drag: None,
            last_scale: 1.0,
            editing_text: None,
            text_size: 24.0,
            text_font: FontChoice::Sans,
            text_bold: false,
            editing_badge: None,
            badge_start: 1,
            badge_border: true,
        }
    }
}

impl Editor {
    /// Réinitialise l'éditeur pour une nouvelle capture (conserve couleur et épaisseur).
    pub fn reset(&mut self) {
        self.annotations.clear();
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.selected = None;
        self.drag = None;
        self.editing_text = None;
        self.editing_badge = None;
    }

    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }

    /// Remplace les annotations (ouverture d'un projet).
    pub fn set_annotations(&mut self, annotations: Vec<Annotation>) {
        self.reset();
        self.annotations = annotations;
        // Reprend le numéro de départ du projet chargé.
        if let Some(min) = self
            .annotations
            .iter()
            .filter_map(|a| match a.shape {
                Shape::Badge { number, .. } => Some(number),
                _ => None,
            })
            .min()
        {
            self.badge_start = min;
        }
    }

    // ----- Groupes de la barre d'outils ---------------------------------------
    // L'application les compose et replie ceux qui ne tiennent pas dans un
    // menu « ⋯ » en bout de barre.

    /// Icônes des outils (toujours visibles).
    pub fn tools_group_ui(&mut self, ui: &mut egui::Ui) {
        let tools: [(Tool, ToolIcon, &str); 6] = [
            (
                Tool::Select,
                widgets::icon_select,
                "Sélection — déplacer / supprimer",
            ),
            (Tool::Badge, widgets::icon_badge, "Pastille numérotée"),
            (Tool::Arrow, widgets::icon_arrow, "Flèche"),
            (Tool::Rect, widgets::icon_rect, "Rectangle"),
            (Tool::Ellipse, widgets::icon_ellipse, "Ellipse"),
            (Tool::Text, widgets::icon_text, "Texte"),
        ];
        for (tool, icon, tip) in tools {
            if widgets::icon_button(ui, self.tool == tool, icon)
                .on_hover_text(tip)
                .clicked()
            {
                self.tool = tool;
            }
        }
    }

    /// Nuancier de couleurs.
    pub fn colors_group_ui(&mut self, ui: &mut egui::Ui) {
        for color in theme::PALETTE {
            if widgets::swatch(ui, color, self.color == color).clicked() {
                self.color = color;
                // Recolore aussi l'annotation sélectionnée.
                if let Some(idx) = self.selected {
                    self.push_undo();
                    if let Some(ann) = self.annotations.get_mut(idx) {
                        ann.color = color;
                    }
                }
            }
        }
    }

    /// Épaisseurs de trait.
    pub fn strokes_group_ui(&mut self, ui: &mut egui::Ui) {
        for width in theme::STROKES {
            let selected = (self.stroke_width - width).abs() < 0.01;
            if widgets::stroke_dot(ui, width, selected).clicked() {
                self.stroke_width = width;
                if let Some(idx) = self.selected {
                    self.push_undo();
                    if let Some(ann) = self.annotations.get_mut(idx) {
                        ann.stroke_width = width;
                    }
                }
            }
        }
    }

    /// Annuler / Rétablir.
    pub fn undo_redo_group_ui(&mut self, ui: &mut egui::Ui) {
        if widgets::icon_button_enabled(ui, !self.undo_stack.is_empty(), widgets::icon_undo)
            .on_hover_text("Annuler (Ctrl+Z)")
            .clicked()
        {
            self.undo();
        }
        if widgets::icon_button_enabled(ui, !self.redo_stack.is_empty(), widgets::icon_redo)
            .on_hover_text("Rétablir (Ctrl+Y)")
            .clicked()
        {
            self.redo();
        }
    }

    /// Options contextuelles (numérotation ou police du texte).
    pub fn options_group_ui(&mut self, ui: &mut egui::Ui) {
        self.badge_options_ui(ui);
        self.text_style_ui(ui);
    }

    /// Largeur estimée du groupe d'options contextuelles (0 = masqué).
    pub fn options_group_width(&self) -> f32 {
        if self.badge_options_visible() {
            300.0
        } else if self.text_options_visible() {
            250.0
        } else {
            0.0
        }
    }

    fn badge_options_visible(&self) -> bool {
        let selected_badge = self
            .selected
            .and_then(|i| self.annotations.get(i))
            .is_some_and(|a| matches!(a.shape, Shape::Badge { .. }));
        self.tool == Tool::Badge || selected_badge
    }

    fn text_options_visible(&self) -> bool {
        let selected_text = self
            .selected
            .and_then(|i| self.annotations.get(i))
            .is_some_and(|a| matches!(a.shape, Shape::Text { .. }));
        self.tool == Tool::Text || selected_text
    }

    /// Options de numérotation, visibles quand l'outil pastille est actif
    /// ou qu'une pastille est sélectionnée.
    fn badge_options_ui(&mut self, ui: &mut egui::Ui) {
        if !self.badge_options_visible() {
            return;
        }

        let start_resp = ui
            .add(
                egui::DragValue::new(&mut self.badge_start)
                    .range(1..=999)
                    .speed(0.2)
                    .prefix("départ "),
            )
            .on_hover_text("Premier numéro de la séquence");
        if start_resp.changed() {
            if !start_resp.dragged() || start_resp.drag_started() {
                self.push_undo();
            }
            self.normalize_badges();
        }

        if ui
            .button("Réordonner")
            .on_hover_text("Renumérote dans l'ordre de lecture (haut → bas, gauche → droite)")
            .clicked()
        {
            self.reorder_badges_reading();
        }

        let border_resp = ui
            .checkbox(&mut self.badge_border, "bordure")
            .on_hover_text("Anneau blanc autour de la pastille");
        if border_resp.changed() {
            // Applique aussi à la pastille sélectionnée.
            if let Some(idx) = self.selected {
                let border = self.badge_border;
                if matches!(
                    self.annotations.get(idx).map(|a| &a.shape),
                    Some(Shape::Badge { .. })
                ) {
                    self.push_undo();
                    if let Some(Annotation {
                        shape: Shape::Badge { bordered, .. },
                        ..
                    }) = self.annotations.get_mut(idx)
                    {
                        *bordered = border;
                    }
                }
            }
        }
    }

    /// Contrôles de police, visibles quand l'outil texte est actif ou
    /// qu'une annotation texte est sélectionnée.
    fn text_style_ui(&mut self, ui: &mut egui::Ui) {
        let selected_text = self
            .selected
            .and_then(|i| self.annotations.get(i))
            .is_some_and(|a| matches!(a.shape, Shape::Text { .. }));
        if self.tool != Tool::Text && !selected_text {
            return;
        }

        let mut changed = false;

        egui::ComboBox::from_id_salt("text_font")
            .selected_text(self.text_font.label())
            .width(72.0)
            .show_ui(ui, |ui| {
                for font in FontChoice::ALL {
                    if ui
                        .selectable_value(&mut self.text_font, font, font.label())
                        .changed()
                    {
                        changed = true;
                    }
                }
            });

        let size_resp = ui.add(
            egui::DragValue::new(&mut self.text_size)
                .range(8.0..=200.0)
                .speed(1.0)
                .suffix(" px"),
        );
        if size_resp.changed() {
            changed = true;
        }

        let bold_resp = ui
            .selectable_label(self.text_bold, egui::RichText::new("B").strong().size(15.0))
            .on_hover_text("Gras");
        if bold_resp.clicked() {
            self.text_bold = !self.text_bold;
            changed = true;
        }

        // Applique le style à l'annotation texte sélectionnée.
        if changed && selected_text
            && let Some(idx) = self.selected {
                // Un seul cran d'undo par « rafale » de changements : on
                // n'empile pas pendant un glissement continu du DragValue.
                if !size_resp.dragged() || size_resp.drag_started() {
                    self.push_undo();
                }
                if let Some(ann) = self.annotations.get_mut(idx)
                    && let Shape::Text {
                        size, font, bold, ..
                    } = &mut ann.shape
                    {
                        *size = self.text_size;
                        *font = self.text_font;
                        *bold = self.text_bold;
                    }
            }
    }

    // ----- Légende -------------------------------------------------------------

    /// Liste des étapes : une ligne par pastille, texte éditable.
    pub fn legend_ui(&mut self, ui: &mut egui::Ui) {
        let seq = self.badge_sequence();
        if seq.is_empty() {
            ui.add_space(8.0);
            ui.weak("Posez des pastilles numérotées pour construire la légende.");
            return;
        }
        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for idx in seq {
                    let (number, color) = match &self.annotations[idx].shape {
                        Shape::Badge { number, .. } => (*number, self.annotations[idx].color),
                        _ => continue,
                    };
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        // Pastille : cliquer la sélectionne sur le canevas.
                        let (rect, resp) =
                            ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::click());
                        ui.painter().circle_filled(rect.center(), 10.0, color);
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            number.to_string(),
                            egui::FontId::proportional(11.0),
                            Color32::WHITE,
                        );
                        if resp.clicked() {
                            self.selected = Some(idx);
                        }
                        let mut gained_focus = false;
                        if let Shape::Badge { label, .. } = &mut self.annotations[idx].shape {
                            let edit = ui.add(
                                egui::TextEdit::singleline(label)
                                    .desired_width(f32::INFINITY)
                                    .hint_text("Texte de l'étape…"),
                            );
                            gained_focus = edit.gained_focus();
                        }
                        if gained_focus {
                            // Un cran d'undo par session d'édition du champ.
                            self.undo_stack.push(self.annotations.clone());
                            self.redo_stack.clear();
                        }
                    });
                }
            });
    }

    /// Légende au format Markdown : `1. texte` par pastille.
    pub fn legend_markdown(&self) -> String {
        self.badge_sequence()
            .iter()
            .filter_map(|&i| match &self.annotations[i].shape {
                Shape::Badge { number, label, .. } => {
                    let text = label.trim();
                    Some(format!(
                        "{number}. {}",
                        if text.is_empty() { "…" } else { text }
                    ))
                }
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn has_badges(&self) -> bool {
        self.annotations
            .iter()
            .any(|a| matches!(a.shape, Shape::Badge { .. }))
    }

    /// Numéros des pastilles présentes, triés.
    pub fn badge_numbers(&self) -> Vec<u32> {
        let mut nums: Vec<u32> = self
            .annotations
            .iter()
            .filter_map(|a| match a.shape {
                Shape::Badge { number, .. } => Some(number),
                _ => None,
            })
            .collect();
        nums.sort_unstable();
        nums
    }

    /// Applique des textes d'étape par numéro de pastille (génération IA).
    pub fn set_badge_labels(&mut self, labels: &[(u32, String)]) {
        if labels.is_empty() {
            return;
        }
        self.push_undo();
        for ann in &mut self.annotations {
            if let Shape::Badge { number, label, .. } = &mut ann.shape
                && let Some((_, text)) = labels.iter().find(|(n, _)| n == number) {
                    *label = text.clone();
                }
        }
    }

    /// Insère des annotations générées (IA), avec numérotation des pastilles
    /// à la suite de la séquence existante.
    pub fn add_annotations(&mut self, mut annotations: Vec<Annotation>) {
        if annotations.is_empty() {
            return;
        }
        self.push_undo();
        let mut next = self
            .annotations
            .iter()
            .filter_map(|a| match a.shape {
                Shape::Badge { number, .. } => Some(number),
                _ => None,
            })
            .max()
            .map_or(self.badge_start, |m| m + 1);
        for ann in &mut annotations {
            if let Shape::Badge { number, .. } = &mut ann.shape {
                *number = next;
                next += 1;
            }
        }
        self.annotations.extend(annotations);
        self.normalize_badges();
        self.selected = None;
    }

    /// Style courant, appliqué aux annotations générées par l'IA.
    pub fn ai_style(&self) -> crate::ai::AiStyle {
        crate::ai::AiStyle {
            color: self.color,
            stroke_width: self.stroke_width,
            text_size: self.text_size,
            font: self.text_font,
            bold: self.text_bold,
            badge_border: self.badge_border,
        }
    }

    /// Indication contextuelle pour la barre d'état.
    pub fn hint(&self) -> &'static str {
        match self.tool {
            Tool::Select => {
                if self.selected.is_some() {
                    "Glissez pour déplacer — Suppr pour supprimer — double-clic : éditer"
                } else {
                    "Cliquez une annotation — double-clic : éditer un texte ou un numéro"
                }
            }
            Tool::Badge => {
                "Cliquez pour poser une pastille — elle s'insère après la pastille sélectionnée"
            }
            Tool::Arrow => "Glissez du point de départ vers la cible — la tête est à l'arrivée",
            Tool::Rect => "Glissez pour encadrer une zone",
            Tool::Ellipse => "Glissez pour encercler une zone",
            Tool::Text => "Cliquez pour placer du texte — Entrée pour valider, Échap pour annuler",
        }
    }

    pub fn annotation_count(&self) -> usize {
        self.annotations.len()
    }

    /// Échelle d'affichage courante (1.0 = 100 %).
    pub fn zoom(&self) -> f32 {
        self.last_scale
    }

    /// Applique les réglages par défaut (couleur, épaisseur).
    pub fn set_defaults(&mut self, color: Color32, stroke_width: f32) {
        self.color = color;
        self.stroke_width = stroke_width;
    }

    /// Affiche l'image et gère le dessin/la sélection des annotations.
    pub fn canvas(&mut self, ui: &mut egui::Ui, texture: &egui::TextureHandle, img_size: Vec2) {
        let avail = ui.available_size();
        let scale = (avail.x / img_size.x)
            .min(avail.y / img_size.y)
            .clamp(0.01, 1.0);
        self.last_scale = scale;
        let (canvas_rect, response) = ui.allocate_exact_size(avail, egui::Sense::click_and_drag());
        let image_rect = Rect::from_center_size(canvas_rect.center(), img_size * scale);
        let t = CanvasTransform {
            origin: image_rect.min,
            scale,
        };

        let painter = ui.painter_at(canvas_rect);
        painter.rect_stroke(
            image_rect.expand(1.0),
            egui::CornerRadius::ZERO,
            Stroke::new(1.0, theme::BORDER),
            egui::StrokeKind::Outside,
        );
        painter.image(
            texture.id(),
            image_rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );

        self.handle_pointer(&response, &t, img_size);
        self.handle_keys(ui.ctx());

        let editing_idx = self.editing_text.as_ref().map(|e| e.index);
        for (i, ann) in self.annotations.iter().enumerate() {
            // L'annotation en cours de saisie est affichée par le TextEdit.
            if Some(i) != editing_idx {
                ann.paint(&painter, &t);
            }
        }
        self.paint_selection(&painter, &t);
        self.paint_preview(&painter, &t);
        self.text_edit_overlay(ui, &t);
        self.badge_edit_overlay(ui, &t);

        match self.tool {
            Tool::Select => {}
            Tool::Text => {
                response.on_hover_cursor(CursorIcon::Text);
            }
            _ => {
                response.on_hover_cursor(CursorIcon::Crosshair);
            }
        }
    }

    // ----- Entrées ---------------------------------------------------------

    fn handle_pointer(&mut self, response: &egui::Response, t: &CanvasTransform, img_size: Vec2) {
        let clamp = |p: Pos2| Pos2::new(p.x.clamp(0.0, img_size.x), p.y.clamp(0.0, img_size.y));
        let pointer = response
            .interact_pointer_pos()
            .map(|p| clamp(t.to_image(p)));

        if response.clicked()
            && let Some(p) = pointer {
                match self.tool {
                    Tool::Badge => self.place_badge(p),
                    Tool::Select => {
                        self.selected = self.hit_test(p, t.scale);
                        self.sync_text_style_from_selection();
                    }
                    Tool::Text => {
                        // Cliquer un texte existant le rouvre, sinon on en crée un.
                        match self.hit_test(p, t.scale) {
                            Some(idx)
                                if matches!(self.annotations[idx].shape, Shape::Text { .. }) =>
                            {
                                self.selected = Some(idx);
                                self.start_text_edit_existing(idx);
                            }
                            _ => self.start_text(p),
                        }
                    }
                    _ => {}
                }
            }

        if response.double_clicked() && self.tool == Tool::Select
            && let Some(p) = pointer
                && let Some(idx) = self.hit_test(p, t.scale) {
                    match self.annotations[idx].shape {
                        Shape::Text { .. } => {
                            self.selected = Some(idx);
                            self.start_text_edit_existing(idx);
                        }
                        Shape::Badge { .. } => {
                            self.selected = Some(idx);
                            self.start_badge_number_edit(idx);
                        }
                        _ => {}
                    }
                }

        if response.drag_started()
            && let Some(p) = pointer {
                match self.tool {
                    Tool::Arrow | Tool::Rect | Tool::Ellipse => {
                        self.drag = Some(Drag::Draw {
                            start: p,
                            current: p,
                        });
                    }
                    Tool::Select => {
                        self.selected = self.hit_test(p, t.scale);
                        self.sync_text_style_from_selection();
                        if self.selected.is_some() {
                            self.drag = Some(Drag::Move {
                                last: p,
                                before: self.annotations.clone(),
                                moved: false,
                            });
                        }
                    }
                    Tool::Badge | Tool::Text => {}
                }
            }

        if response.dragged()
            && let Some(p) = pointer {
                match &mut self.drag {
                    Some(Drag::Draw { current, .. }) => *current = p,
                    Some(Drag::Move { last, moved, .. }) => {
                        let delta = p - *last;
                        if delta != Vec2::ZERO {
                            if let Some(idx) = self.selected {
                                self.annotations[idx].translate(delta);
                                *moved = true;
                            }
                            *last = p;
                        }
                    }
                    None => {}
                }
            }

        if response.drag_stopped() {
            match self.drag.take() {
                Some(Drag::Draw { start, current }) => self.commit_draw(start, current),
                Some(Drag::Move { before, moved: true, .. }) => {
                    self.undo_stack.push(before);
                    self.redo_stack.clear();
                }
                _ => {}
            }
        }
    }

    fn handle_keys(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let (undo, redo, delete) = ctx.input(|i| {
            (
                i.modifiers.ctrl && i.key_pressed(egui::Key::Z),
                i.modifiers.ctrl && i.key_pressed(egui::Key::Y),
                i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace),
            )
        });
        if undo {
            self.undo();
        }
        if redo {
            self.redo();
        }
        if delete {
            self.delete_selected();
        }
    }

    // ----- Mutations -------------------------------------------------------

    fn place_badge(&mut self, p: Pos2) {
        self.push_undo();
        // Insertion après la pastille sélectionnée, sinon en fin de séquence.
        let after = self
            .selected
            .and_then(|i| self.annotations.get(i))
            .and_then(|a| match a.shape {
                Shape::Badge { number, .. } => Some(number),
                _ => None,
            });
        let number = match after {
            Some(k) => {
                for ann in &mut self.annotations {
                    if let Shape::Badge { number, .. } = &mut ann.shape
                        && *number > k {
                            *number += 1;
                        }
                }
                k + 1
            }
            None => self
                .annotations
                .iter()
                .filter_map(|a| match a.shape {
                    Shape::Badge { number, .. } => Some(number),
                    _ => None,
                })
                .max()
                .map_or(self.badge_start, |m| m + 1),
        };
        self.annotations.push(Annotation {
            shape: Shape::Badge {
                center: p,
                number,
                label: String::new(),
                bordered: self.badge_border,
            },
            color: self.color,
            stroke_width: self.stroke_width,
        });
        self.normalize_badges();
        // La nouvelle pastille devient la sélection : les suivantes
        // s'insèrent naturellement à sa suite.
        self.selected = Some(self.annotations.len() - 1);
    }

    /// Indices des pastilles, triés par numéro croissant.
    fn badge_sequence(&self) -> Vec<usize> {
        let mut idxs: Vec<usize> = self
            .annotations
            .iter()
            .enumerate()
            .filter(|(_, a)| matches!(a.shape, Shape::Badge { .. }))
            .map(|(i, _)| i)
            .collect();
        idxs.sort_by_key(|&i| match self.annotations[i].shape {
            Shape::Badge { number, .. } => number,
            _ => 0,
        });
        idxs
    }

    /// Réassigne des numéros consécutifs à partir de `badge_start`,
    /// en respectant l'ordre actuel des numéros.
    fn normalize_badges(&mut self) {
        for (rank, idx) in self.badge_sequence().into_iter().enumerate() {
            if let Shape::Badge { number, .. } = &mut self.annotations[idx].shape {
                *number = self.badge_start + rank as u32;
            }
        }
    }

    /// Donne le numéro `target` à la pastille `idx` en décalant les autres.
    fn set_badge_number(&mut self, idx: usize, target: u32) {
        let Some(Shape::Badge { number: old, .. }) = self.annotations.get(idx).map(|a| &a.shape)
        else {
            return;
        };
        let old = *old;
        let count = self.badge_count();
        if count == 0 {
            return;
        }
        let target = target.clamp(self.badge_start, self.badge_start + count - 1);
        if target == old {
            return;
        }
        self.push_undo();
        for (i, ann) in self.annotations.iter_mut().enumerate() {
            if i == idx {
                continue;
            }
            if let Shape::Badge { number, .. } = &mut ann.shape {
                if target > old {
                    if *number > old && *number <= target {
                        *number -= 1;
                    }
                } else if *number >= target && *number < old {
                    *number += 1;
                }
            }
        }
        if let Shape::Badge { number, .. } = &mut self.annotations[idx].shape {
            *number = target;
        }
        self.normalize_badges();
    }

    /// Renumérote les pastilles dans l'ordre de lecture :
    /// regroupées par lignes (haut → bas), puis de gauche à droite.
    fn reorder_badges_reading(&mut self) {
        // (index, centre, diamètre) de chaque pastille.
        let mut badges: Vec<(usize, Pos2, f32)> = self
            .annotations
            .iter()
            .enumerate()
            .filter_map(|(i, a)| match a.shape {
                Shape::Badge { center, .. } => Some((
                    i,
                    center,
                    crate::annotation::badge_radius(a.stroke_width) * 2.0,
                )),
                _ => None,
            })
            .collect();
        if badges.len() < 2 {
            return;
        }
        self.push_undo();
        // Bande verticale : deux pastilles dont les centres sont à moins
        // d'un diamètre l'une de l'autre appartiennent à la même ligne.
        let band = badges.iter().map(|(_, _, d)| d).sum::<f32>() / badges.len() as f32;
        badges.sort_by(|a, b| a.1.y.total_cmp(&b.1.y));
        let mut rows: Vec<(f32, Vec<(usize, Pos2)>)> = Vec::new();
        for (idx, center, _) in badges {
            match rows.last_mut() {
                Some((anchor, row)) if (center.y - *anchor).abs() <= band => {
                    row.push((idx, center));
                }
                _ => rows.push((center.y, vec![(idx, center)])),
            }
        }
        let mut rank = 0u32;
        for (_, mut row) in rows {
            row.sort_by(|a, b| a.1.x.total_cmp(&b.1.x));
            for (idx, _) in row {
                if let Shape::Badge { number, .. } = &mut self.annotations[idx].shape {
                    *number = self.badge_start + rank;
                }
                rank += 1;
            }
        }
    }

    fn start_badge_number_edit(&mut self, idx: usize) {
        let Some(Shape::Badge { number, .. }) = self.annotations.get(idx).map(|a| &a.shape) else {
            return;
        };
        self.editing_badge = Some(BadgeEditState {
            index: idx,
            buffer: number.to_string(),
            request_focus: true,
        });
    }

    /// Champ de saisie du numéro, affiché sur la pastille en cours d'édition.
    fn badge_edit_overlay(&mut self, ui: &mut egui::Ui, t: &CanvasTransform) {
        let Some(edit) = &mut self.editing_badge else {
            return;
        };
        let Some(ann) = self.annotations.get(edit.index) else {
            self.editing_badge = None;
            return;
        };
        let Shape::Badge { center, .. } = &ann.shape else {
            self.editing_badge = None;
            return;
        };
        let screen = t.to_screen(*center);

        let mut done = false;
        let mut cancel = false;
        egui::Area::new(egui::Id::new("badge_number_edit"))
            .fixed_pos(screen + egui::vec2(14.0, -14.0))
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |aui| {
                let resp = aui.add(
                    egui::TextEdit::singleline(&mut edit.buffer)
                        .font(egui::FontId::proportional(14.0))
                        .desired_width(44.0),
                );
                if edit.request_focus {
                    resp.request_focus();
                    edit.request_focus = false;
                }
                if aui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    cancel = true;
                } else if resp.lost_focus() {
                    done = true;
                }
            });
        if cancel {
            self.editing_badge = None;
        } else if done
            && let Some(edit) = self.editing_badge.take()
                && let Ok(n) = edit.buffer.trim().parse::<u32>() {
                    self.set_badge_number(edit.index, n);
                }
    }

    fn commit_draw(&mut self, start: Pos2, end: Pos2) {
        let shape = match self.tool {
            Tool::Arrow if start.distance(end) >= 5.0 => Some(Shape::Arrow {
                from: start,
                to: end,
            }),
            Tool::Rect | Tool::Ellipse => {
                let rect = Rect::from_two_pos(start, end);
                if rect.width() >= 3.0 && rect.height() >= 3.0 {
                    Some(if self.tool == Tool::Rect {
                        Shape::Rect { rect }
                    } else {
                        Shape::Ellipse { rect }
                    })
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(shape) = shape {
            self.push_undo();
            self.annotations.push(Annotation {
                shape,
                color: self.color,
                stroke_width: self.stroke_width,
            });
        }
    }

    // ----- Texte -------------------------------------------------------------

    /// Reflète le style de l'annotation sélectionnée dans les contrôles
    /// de la barre d'outils.
    fn sync_text_style_from_selection(&mut self) {
        if let Some(ann) = self.selected.and_then(|i| self.annotations.get(i)) {
            match &ann.shape {
                Shape::Text {
                    size, font, bold, ..
                } => {
                    self.text_size = *size;
                    self.text_font = *font;
                    self.text_bold = *bold;
                }
                Shape::Badge { bordered, .. } => {
                    self.badge_border = *bordered;
                }
                _ => {}
            }
        }
    }

    fn start_text(&mut self, p: Pos2) {
        self.commit_text_edit();
        self.push_undo();
        self.annotations.push(Annotation {
            shape: Shape::Text {
                pos: p,
                text: String::new(),
                size: self.text_size,
                font: self.text_font,
                bold: self.text_bold,
            },
            color: self.color,
            stroke_width: self.stroke_width,
        });
        self.selected = None;
        self.editing_text = Some(TextEditState {
            index: self.annotations.len() - 1,
            buffer: String::new(),
            original: None,
            just_created: true,
            request_focus: true,
        });
    }

    fn start_text_edit_existing(&mut self, idx: usize) {
        self.commit_text_edit();
        if let Some(ann) = self.annotations.get(idx)
            && let Shape::Text { text, .. } = &ann.shape {
                self.editing_text = Some(TextEditState {
                    index: idx,
                    buffer: text.clone(),
                    original: Some(text.clone()),
                    just_created: false,
                    request_focus: true,
                });
                self.sync_text_style_from_selection();
            }
    }

    fn commit_text_edit(&mut self) {
        let Some(edit) = self.editing_text.take() else {
            return;
        };
        let trimmed = edit.buffer.trim().to_string();
        if edit.just_created {
            if trimmed.is_empty() {
                // Création avortée : on retire l'annotation et son cran d'undo.
                if edit.index < self.annotations.len() {
                    self.annotations.remove(edit.index);
                }
                self.undo_stack.pop();
            } else if let Some(ann) = self.annotations.get_mut(edit.index)
                && let Shape::Text { text, .. } = &mut ann.shape {
                    *text = trimmed;
                }
        } else if trimmed.is_empty() {
            // Texte vidé : suppression de l'annotation.
            self.push_undo();
            if edit.index < self.annotations.len() {
                self.annotations.remove(edit.index);
            }
            self.selected = None;
        } else if edit.original.as_deref() != Some(trimmed.as_str()) {
            self.push_undo();
            if let Some(ann) = self.annotations.get_mut(edit.index)
                && let Shape::Text { text, .. } = &mut ann.shape {
                    *text = trimmed;
                }
        }
    }

    fn cancel_text_edit(&mut self) {
        let Some(edit) = self.editing_text.take() else {
            return;
        };
        if edit.just_created {
            if edit.index < self.annotations.len() {
                self.annotations.remove(edit.index);
            }
            self.undo_stack.pop();
        }
        // Édition d'un texte existant : rien n'a encore été modifié.
    }

    /// Zone de saisie affichée par-dessus le canevas, à la position du texte.
    fn text_edit_overlay(&mut self, ui: &mut egui::Ui, t: &CanvasTransform) {
        let Some(edit) = &mut self.editing_text else {
            return;
        };
        let Some(ann) = self.annotations.get(edit.index) else {
            self.editing_text = None;
            return;
        };
        let Shape::Text {
            pos,
            size,
            font,
            bold,
            ..
        } = &ann.shape
        else {
            self.editing_text = None;
            return;
        };
        let screen_pos = t.to_screen(*pos);
        let font_id = egui::FontId::new(*size * t.scale, font.family(*bold));
        let color = ann.color;

        let mut commit = false;
        let mut cancel = false;
        egui::Area::new(egui::Id::new("annotation_text_edit"))
            // Léger décalage pour compenser la marge interne du TextEdit.
            .fixed_pos(screen_pos - egui::vec2(5.0, 3.0))
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |aui| {
                let resp = aui.add(
                    egui::TextEdit::singleline(&mut edit.buffer)
                        .font(font_id)
                        .text_color(color)
                        .desired_width(240.0),
                );
                if edit.request_focus {
                    resp.request_focus();
                    edit.request_focus = false;
                }
                if aui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    cancel = true;
                } else if resp.lost_focus() {
                    commit = true;
                }
            });
        if cancel {
            self.cancel_text_edit();
        } else if commit {
            self.commit_text_edit();
        }
    }

    fn delete_selected(&mut self) {
        if let Some(idx) = self.selected.take()
            && idx < self.annotations.len() {
                self.push_undo();
                self.annotations.remove(idx);
                self.renumber_badges();
            }
    }

    fn renumber_badges(&mut self) {
        self.normalize_badges();
    }

    fn badge_count(&self) -> u32 {
        self.annotations
            .iter()
            .filter(|a| matches!(a.shape, Shape::Badge { .. }))
            .count() as u32
    }

    fn hit_test(&self, p: Pos2, scale: f32) -> Option<usize> {
        let tol = HIT_TOLERANCE / scale;
        self.annotations.iter().rposition(|a| a.hit(p, tol))
    }

    fn push_undo(&mut self) {
        self.undo_stack.push(self.annotations.clone());
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack
                .push(std::mem::replace(&mut self.annotations, prev));
            self.selected = None;
            self.drag = None;
            self.editing_text = None;
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack
                .push(std::mem::replace(&mut self.annotations, next));
            self.selected = None;
            self.drag = None;
            self.editing_text = None;
        }
    }

    // ----- Rendu -----------------------------------------------------------

    fn paint_selection(&self, painter: &egui::Painter, t: &CanvasTransform) {
        let Some(idx) = self.selected else { return };
        let Some(ann) = self.annotations.get(idx) else {
            return;
        };
        let bb = ann.bounding_rect();
        let screen = Rect::from_min_max(t.to_screen(bb.min), t.to_screen(bb.max)).expand(6.0);
        let corners = [
            screen.left_top(),
            screen.right_top(),
            screen.right_bottom(),
            screen.left_bottom(),
            screen.left_top(),
        ];
        for shape in egui::Shape::dashed_line(
            &corners,
            Stroke::new(1.5, Color32::from_rgb(0x3B, 0x82, 0xF6)),
            5.0,
            4.0,
        ) {
            painter.add(shape);
        }
    }

    fn paint_preview(&self, painter: &egui::Painter, t: &CanvasTransform) {
        let Some(Drag::Draw { start, current }) = &self.drag else {
            return;
        };
        let shape = match self.tool {
            Tool::Arrow => Shape::Arrow {
                from: *start,
                to: *current,
            },
            Tool::Rect => Shape::Rect {
                rect: Rect::from_two_pos(*start, *current),
            },
            Tool::Ellipse => Shape::Ellipse {
                rect: Rect::from_two_pos(*start, *current),
            },
            _ => return,
        };
        Annotation {
            shape,
            color: self.color,
            stroke_width: self.stroke_width,
        }
        .paint(painter, t);
    }
}
