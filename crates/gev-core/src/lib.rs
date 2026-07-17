//! Cœur partagé GEVSHOT / GEVSTUDIO : modèle d'annotations, format projet
//! `.gev`, aplatissement des annotations sur les pixels, polices d'export.
//! Aucune dépendance GUI — uniquement les types math/couleur (`emath`,
//! `ecolor`, identiques à ceux ré-exportés par egui).

pub mod fonts;
pub mod model;
pub mod project;
pub mod render;
