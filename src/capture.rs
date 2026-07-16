use xcap::{Monitor, Window};

/// Capture l'écran principal (ou le premier écran disponible).
pub fn capture_primary() -> Result<image::RgbaImage, String> {
    let mut monitors =
        Monitor::all().map_err(|e| format!("Énumération des écrans impossible : {e}"))?;
    if monitors.is_empty() {
        return Err("Aucun écran détecté".to_string());
    }
    let idx = monitors
        .iter()
        .position(|m| m.is_primary().unwrap_or(false))
        .unwrap_or(0);
    let monitor = monitors.swap_remove(idx);
    monitor
        .capture_image()
        .map_err(|e| format!("Capture impossible : {e}"))
}

/// Capture la fenêtre actuellement au premier plan, bords propres.
pub fn capture_focused_window() -> Result<image::RgbaImage, String> {
    let windows =
        Window::all().map_err(|e| format!("Énumération des fenêtres impossible : {e}"))?;
    let focused = windows
        .into_iter()
        .find(|w| w.is_focused().unwrap_or(false) && !w.is_minimized().unwrap_or(true))
        .ok_or("Aucune fenêtre active détectée")?;
    focused
        .capture_image()
        .map_err(|e| format!("Capture de la fenêtre impossible : {e}"))
}

/// Fenêtre candidate à la capture, pour le sélecteur.
#[derive(Clone)]
pub struct WindowInfo {
    pub id: u32,
    pub app_name: String,
    pub title: String,
}

/// Liste les fenêtres capturables : visibles, titrées, de taille raisonnable,
/// en excluant celles de cette application.
pub fn list_windows() -> Result<Vec<WindowInfo>, String> {
    let own_pid = std::process::id();
    let windows =
        Window::all().map_err(|e| format!("Énumération des fenêtres impossible : {e}"))?;
    let mut infos: Vec<WindowInfo> = windows
        .iter()
        .filter(|w| {
            !w.is_minimized().unwrap_or(true)
                && w.pid().map(|p| p != own_pid).unwrap_or(false)
                && w.width().unwrap_or(0) >= 120
                && w.height().unwrap_or(0) >= 80
        })
        .filter_map(|w| {
            let title = w.title().ok()?;
            if title.trim().is_empty() {
                return None;
            }
            Some(WindowInfo {
                id: w.id().ok()?,
                app_name: w.app_name().unwrap_or_default(),
                title,
            })
        })
        .collect();
    infos.dedup_by_key(|w| w.id);
    if infos.is_empty() {
        return Err("Aucune fenêtre capturable détectée".to_string());
    }
    Ok(infos)
}

/// Capture une fenêtre précise, choisie dans le sélecteur.
pub fn capture_window_by_id(id: u32) -> Result<image::RgbaImage, String> {
    let windows =
        Window::all().map_err(|e| format!("Énumération des fenêtres impossible : {e}"))?;
    let window = windows
        .into_iter()
        .find(|w| w.id().map(|w_id| w_id == id).unwrap_or(false))
        .ok_or("Cette fenêtre n'existe plus")?;
    if window.is_minimized().unwrap_or(false) {
        return Err("Cette fenêtre est réduite — restaurez-la avant la capture".to_string());
    }
    window
        .capture_image()
        .map_err(|e| format!("Capture de la fenêtre impossible : {e}"))
}
