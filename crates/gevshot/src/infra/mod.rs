//! Couche infrastructure : tout ce qui touche au système ou au réseau —
//! capture d'écran, fichiers, presse-papiers, configuration, polices,
//! barre système, API d'IA.

pub mod ai;
pub mod capture;
pub mod config;
pub mod export;
pub mod fonts;
pub use gev_core::project;
pub mod tray;
