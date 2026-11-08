#![cfg(test)]

use iced::alignment::{Horizontal, Vertical};

use crate::contracts::settings::keys::settings_window as key;
use crate::frontend::settings::test_source::FakeSettingsSource;
use crate::frontend::ui::{SheetPosition, SheetWindow};

use super::{
    picker_layout_studio_align, picker_layout_studio_height, picker_layout_studio_inset,
    picker_layout_studio_on_right, picker_layout_studio_width, sheet_window,
};

#[test]
fn sheet_window_reads_the_settings_window_keys() {
    let source = FakeSettingsSource::default()
        .with_text(key::WIDTH_PERCENT, "60")
        .with_text(key::HEIGHT_PERCENT, "150")
        .with_text(key::SCALE, "0.8")
        .with_text(key::POSITION, "right");
    let window = sheet_window(&source, 1.25);
    assert_eq!(window, SheetWindow::new(0.8, 0.6, 1.0, SheetPosition::Right));
    assert_eq!(window.dims((1000.0, 800.0)), (600.0, 778.0));
}

#[test]
fn sheet_window_defaults_follow_the_ui_scale() {
    let window = sheet_window(&FakeSettingsSource::default(), 1.25);
    assert_eq!(window, SheetWindow::automatic(1.25));
    let tiny = FakeSettingsSource::default()
        .with_text(key::SCALE, "0.2")
        .with_text(key::WIDTH_PERCENT, "10");
    assert_eq!(sheet_window(&tiny, 1.0), SheetWindow::automatic(1.0));
    let huge = FakeSettingsSource::default().with_text(key::SCALE, "9");
    assert_eq!(sheet_window(&huge, 1.0).scale(), 2.0);
}

#[test]
fn layout_studio_keeps_its_automatic_size() {
    let window = SheetWindow::automatic(1.0);
    assert_eq!(picker_layout_studio_width(1280.0, &window), 440.0);
    assert_eq!(picker_layout_studio_height(720.0, &window), 684.0);
    assert_eq!(picker_layout_studio_inset(1280.0, &window), 476.0);
    assert!(!picker_layout_studio_on_right(&window, false));
    assert!(picker_layout_studio_on_right(&window, true));
    assert_eq!(picker_layout_studio_align(&window, false), (Horizontal::Left, Vertical::Center));
}

#[test]
fn layout_studio_follows_the_window_but_leaves_the_picker_visible() {
    let narrow = SheetWindow::new(1.0, 0.25, 0.5, SheetPosition::Bottom);
    assert_eq!(picker_layout_studio_width(1280.0, &narrow), 320.0);
    assert_eq!(picker_layout_studio_height(720.0, &narrow), 360.0);
    assert_eq!(picker_layout_studio_align(&narrow, false), (Horizontal::Left, Vertical::Bottom));
    let wide = SheetWindow::new(1.0, 0.9, 0.0, SheetPosition::Right);
    assert_eq!(picker_layout_studio_width(1280.0, &wide), 1280.0 * 0.48);
    assert!(picker_layout_studio_on_right(&wide, false));
    assert_eq!(picker_layout_studio_align(&wide, true), (Horizontal::Right, Vertical::Center));
    let left = SheetWindow::new(1.5, 0.0, 0.0, SheetPosition::Left);
    assert!(!picker_layout_studio_on_right(&left, true));
    assert_eq!(picker_layout_studio_width(1920.0, &left), 660.0);
    assert_eq!(picker_layout_studio_inset(1920.0, &left), 714.0);
}
