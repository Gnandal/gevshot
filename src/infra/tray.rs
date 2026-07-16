use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem},
};

pub enum TrayCommand {
    /// Capture d'une région (mode par défaut).
    Capture,
    /// Capture d'une région après 3 secondes.
    CaptureDelayed,
    /// Capture de l'écran entier, sans sélection.
    CaptureScreen,
    /// Capture de la fenêtre active.
    CaptureWindow,
    /// Ouvre le sélecteur de fenêtre à capturer.
    PickWindow,
    OpenEditor,
    Quit,
}

pub struct Tray {
    _tray: TrayIcon,
    capture_id: MenuId,
    capture_delayed_id: MenuId,
    capture_screen_id: MenuId,
    capture_window_id: MenuId,
    pick_window_id: MenuId,
    open_id: MenuId,
    quit_id: MenuId,
}

impl Tray {
    pub fn new() -> Result<Self, String> {
        let menu = Menu::new();
        let capture = MenuItem::new("Capturer une région\tCtrl+Shift+S", true, None);
        let capture_delayed = MenuItem::new("Capturer une région dans 3 s", true, None);
        let capture_screen = MenuItem::new("Capturer l'écran entier", true, None);
        let capture_window = MenuItem::new("Capturer la fenêtre active", true, None);
        let pick_window = MenuItem::new("Choisir une fenêtre à capturer…", true, None);
        let open = MenuItem::new("Ouvrir l'éditeur", true, None);
        let quit = MenuItem::new("Quitter", true, None);
        menu.append_items(&[
            &capture,
            &capture_delayed,
            &capture_screen,
            &capture_window,
            &pick_window,
            &PredefinedMenuItem::separator(),
            &open,
            &PredefinedMenuItem::separator(),
            &quit,
        ])
        .map_err(|e| format!("Menu de la barre système impossible : {e}"))?;
        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("GEVSHOT — capture d'écran annotée")
            .with_icon(make_icon())
            .build()
            .map_err(|e| format!("Icône de la barre système impossible : {e}"))?;
        Ok(Self {
            _tray: tray,
            capture_id: capture.id().clone(),
            capture_delayed_id: capture_delayed.id().clone(),
            capture_screen_id: capture_screen.id().clone(),
            capture_window_id: capture_window.id().clone(),
            pick_window_id: pick_window.id().clone(),
            open_id: open.id().clone(),
            quit_id: quit.id().clone(),
        })
    }

    /// Dépile les événements du menu et de l'icône (appelé à chaque frame).
    pub fn poll(&self) -> Option<TrayCommand> {
        if let Ok(ev) = MenuEvent::receiver().try_recv() {
            let id = ev.id();
            if *id == self.capture_id {
                return Some(TrayCommand::Capture);
            }
            if *id == self.capture_delayed_id {
                return Some(TrayCommand::CaptureDelayed);
            }
            if *id == self.capture_screen_id {
                return Some(TrayCommand::CaptureScreen);
            }
            if *id == self.capture_window_id {
                return Some(TrayCommand::CaptureWindow);
            }
            if *id == self.pick_window_id {
                return Some(TrayCommand::PickWindow);
            }
            if *id == self.open_id {
                return Some(TrayCommand::OpenEditor);
            }
            if *id == self.quit_id {
                return Some(TrayCommand::Quit);
            }
        }
        if let Ok(TrayIconEvent::DoubleClick { .. }) = TrayIconEvent::receiver().try_recv() {
            return Some(TrayCommand::OpenEditor);
        }
        None
    }
}

/// Icône de la barre système : la même que celle de l'application.
fn make_icon() -> Icon {
    const S: u32 = 32;
    Icon::from_rgba(crate::appicon::icon_rgba(S), S, S).expect("icône générée invalide")
}
