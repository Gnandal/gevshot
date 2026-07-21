//! Flux de capture : déclenchement (barre système, raccourci, boutons),
//! délai de masquage, prise de vue selon le mode, entrée en édition.

use std::time::{Duration, Instant};

use eframe::egui;
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};

use super::{Capture, CaptureMode, GevshotApp, Stage};
use crate::infra::tray::{Tray, TrayCommand};

impl GevshotApp {
    /// Dépile les commandes de la barre système et du raccourci global.
    pub(super) fn poll_integrations(&mut self, ctx: &egui::Context) {
        if let Some(cmd) = self.tray.as_ref().and_then(Tray::poll) {
            match cmd {
                TrayCommand::Capture => self.start_capture(ctx),
                TrayCommand::CaptureDelayed => {
                    self.start_capture_mode(ctx, CaptureMode::Region, 3000);
                }
                TrayCommand::CaptureScreen => {
                    self.start_capture_mode(ctx, CaptureMode::FullScreen, 0);
                }
                TrayCommand::CaptureWindow => {
                    self.start_capture_mode(ctx, CaptureMode::Window, 0);
                }
                TrayCommand::PickWindow => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    self.open_window_picker();
                }
                TrayCommand::OpenEditor => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                }
                TrayCommand::Quit => {
                    self.quitting = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
        if let Ok(ev) = GlobalHotKeyEvent::receiver().try_recv()
            && Some(ev.id()) == self.hotkey_id
            && ev.state() == HotKeyState::Pressed
            && self.pending_capture.is_none()
            && !matches!(self.stage, Stage::Selecting { .. })
        {
            self.start_capture(ctx);
        }
    }

    pub(super) fn start_capture(&mut self, ctx: &egui::Context) {
        self.start_capture_mode(ctx, CaptureMode::Region, 0);
    }

    /// Lance une capture : masque la fenêtre, attend le délai (masquage +
    /// minuteur optionnel), puis prend la vue selon le mode.
    pub(super) fn start_capture_mode(
        &mut self,
        ctx: &egui::Context,
        mode: CaptureMode,
        extra_delay_ms: u64,
    ) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        self.pending_mode = mode;
        self.pending_delay_ms = self.settings.capture_delay_ms + extra_delay_ms;
        self.pending_capture = Some(Instant::now());
        self.error = None;
        self.status = None;
    }

    pub(super) fn poll_pending_capture(&mut self, ctx: &egui::Context) {
        let Some(started) = self.pending_capture else {
            return;
        };
        if started.elapsed() < Duration::from_millis(self.pending_delay_ms) {
            ctx.request_repaint_after(Duration::from_millis(50));
            return;
        }
        self.pending_capture = None;
        match self.pending_mode {
            CaptureMode::Region => match crate::infra::capture::capture_primary() {
                Ok(image) => {
                    let full = Capture::from_image(ctx, image);
                    let prev = match std::mem::replace(&mut self.stage, Stage::Empty) {
                        Stage::Editing { capture } => Some(capture),
                        _ => None,
                    };
                    self.stage = Stage::Selecting { full, prev };
                    self.selection = None;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Decorations(false));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                }
                Err(err) => self.error = Some(err),
            },
            CaptureMode::FullScreen => match crate::infra::capture::capture_primary() {
                Ok(image) => self.enter_editing(ctx, image),
                Err(err) => self.error = Some(err),
            },
            CaptureMode::Window => match crate::infra::capture::capture_focused_window() {
                Ok(image) => self.enter_editing(ctx, image),
                Err(err) => self.error = Some(err),
            },
            CaptureMode::WindowById(id) => match crate::infra::capture::capture_window_by_id(id) {
                Ok(image) => self.enter_editing(ctx, image),
                Err(err) => self.error = Some(err),
            },
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    /// Passe directement en édition avec l'image donnée (plein écran, fenêtre).
    pub(super) fn enter_editing(&mut self, ctx: &egui::Context, image: image::RgbaImage) {
        let capture = Capture::from_image(ctx, image);
        self.stage = Stage::Editing { capture };
        if !std::mem::take(&mut self.keep_annotations) {
            self.editor.reset();
        }
        self.doc_name = crate::infra::export::next_auto_name(&self.save_dir());
        self.update_title(ctx);
        self.copy_to_clipboard();
    }
}
