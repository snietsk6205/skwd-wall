#![cfg(test)]

use crate::frontend::ui::{SheetPosition, SheetWindow};

fn tile_is_wide(viewport: (f32, f32), scale: f32) -> bool {
    super::tile_is_wide(SheetWindow::automatic(scale).dims(viewport).0, scale)
}

#[test]
fn display_tiles_go_wide_only_with_room() {
    assert!(!tile_is_wide((1280.0, 800.0), 1.0));
    assert!(tile_is_wide(crate::frontend::ui::MIN_VIEWPORT, 1.0));
    assert!(tile_is_wide((1920.0, 1080.0), 1.0));
    assert!(tile_is_wide((2560.0, 1440.0), 1.0));
    assert!(!tile_is_wide((1920.0, 1080.0), 1.5));
}

#[test]
fn display_tiles_follow_the_window_width() {
    let screen = (1920.0, 1080.0);
    assert!(tile_is_wide(screen, 1.0));
    let narrow = SheetWindow::new(1.0, 0.4, 0.0, SheetPosition::Center);
    assert!(!super::tile_is_wide(narrow.dims(screen).0, 1.0));
}
