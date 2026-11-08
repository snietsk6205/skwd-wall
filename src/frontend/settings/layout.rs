use iced::alignment::{Horizontal, Vertical};

use crate::contracts::settings::SettingsSource;
use crate::frontend::ui::{SheetPosition, SheetWindow};

const PICKER_LAYOUT_STUDIO_WIDTH: f32 = 440.0;
/// The layout studio never covers more than this share of the screen, so the
/// live picker beside it stays visible.
const STUDIO_MAX_SHARE: f32 = 0.48;
/// Gap between the layout studio and the screen edge it is docked to.
pub const STUDIO_EDGE_GAP: f32 = 18.0;

/// Settings → Picker → Settings window: the size, place and scale that the
/// Settings window and every panel over the picker share.
pub fn sheet_window(cfg: &dyn SettingsSource, ui_scale: f32) -> SheetWindow {
    use crate::contracts::settings::keys::settings_window as key;
    let share = |path: &str| {
        let value = cfg.number(path);
        if value.is_finite() { (value / 100.0) as f32 } else { 0.0 }
    };
    let scale = cfg.number(key::SCALE);
    let scale = if scale.is_finite() && scale >= 0.5 { scale.min(2.0) as f32 } else { ui_scale };
    SheetWindow::new(
        scale,
        share(key::WIDTH_PERCENT),
        share(key::HEIGHT_PERCENT),
        SheetPosition::parse(&cfg.text(key::POSITION)),
    )
}

pub fn picker_layout_studio_width(viewport_width: f32, window: &SheetWindow) -> f32 {
    let scale = window.scale();
    let available = (viewport_width - 36.0 * scale).max(320.0);
    let cap = (viewport_width * STUDIO_MAX_SHARE).max(320.0);
    let auto = (PICKER_LAYOUT_STUDIO_WIDTH * scale.max(0.8)).min(available).min(cap);
    window.width_or(viewport_width, auto).min(available).min(cap)
}

pub fn picker_layout_studio_height(viewport_height: f32, window: &SheetWindow) -> f32 {
    let auto = (viewport_height - 36.0 * window.scale()).max(360.0);
    window.height_or(viewport_height, auto)
}

/// The studio is a side panel: Left and Right pick the screen side, the other
/// positions keep it at the start of the reading direction.
pub fn picker_layout_studio_on_right(window: &SheetWindow, rtl: bool) -> bool {
    match window.position() {
        SheetPosition::Left => false,
        SheetPosition::Right => true,
        SheetPosition::Center | SheetPosition::Top | SheetPosition::Bottom => rtl,
    }
}

pub fn picker_layout_studio_align(window: &SheetWindow, rtl: bool) -> (Horizontal, Vertical) {
    let horizontal = if picker_layout_studio_on_right(window, rtl) {
        Horizontal::Right
    } else {
        Horizontal::Left
    };
    (horizontal, window.align_y())
}

/// How far the picker moves aside while the studio is open.
pub fn picker_layout_studio_inset(viewport_width: f32, window: &SheetWindow) -> f32 {
    picker_layout_studio_width(viewport_width, window) + 2.0 * STUDIO_EDGE_GAP * window.scale()
}

#[cfg(test)]
mod tests;
