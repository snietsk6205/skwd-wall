use super::*;

#[test]
fn tracker_spans_rail() {
    assert_eq!(tracker_position(0, 11), 0.0);
    assert_eq!(tracker_position(10, 11), 1.0);
    assert_eq!(tracker_position(99, 11), 1.0);
}

#[test]
fn button_wipe_diagonal() {
    assert_eq!(crate::frontend::ui::folio_diagonal_edges(120.0, 30.0, -1.0), (0.0, 0.0));
    assert_eq!(crate::frontend::ui::folio_diagonal_edges(120.0, 30.0, 0.0), (0.0, 0.0));
    let middle = crate::frontend::ui::folio_diagonal_edges(120.0, 30.0, 0.5);
    assert!((middle.0 - 38.4).abs() < 0.001);
    assert!((middle.1 - 81.6).abs() < 0.001);
    assert_eq!(crate::frontend::ui::folio_diagonal_edges(120.0, 30.0, 1.0), (120.0, 120.0));
    assert_eq!(crate::frontend::ui::folio_diagonal_edges(120.0, 30.0, 2.0), (120.0, 120.0));
}

#[test]
fn glyph_fade_opaque() {
    assert_eq!(super::GLYPH_FADE, 1.0);
}

#[test]
fn small_type_floor() {
    assert_eq!(crate::frontend::ui::TYPE_SMALL, 11.0);
}

#[test]
fn editor_bars_match_kind() {
    let source = include_str!("../folio.rs");
    assert!(source.contains("control.is_inline_editor()"));
    assert!(source.contains("crate::frontend::ui::folio_inline_bar("));
    assert!(source.contains("crate::frontend::ui::folio_stack_bar("));
    assert!(source.contains("SettingsMsg::ToggleBar(id, index)"));
}

#[test]
fn inline_vs_group_editors() {
    let inline = [
        Control::Number { key: "number".into(), path: String::new(), unit: "px" },
        Control::TextField { key: "text".into(), path: String::new(), placeholder: "path" },
        Control::KeyBinding { key: "key".into(), path: String::new(), default: "ctrl+f" },
    ];
    assert!(inline.iter().all(Control::is_inline_editor));
    let grouped = Control::MotionWeights {
        weights: vec![(
            String::from("fast"),
            String::from("motion.fast"),
            ActionId::ResetMotionFast,
        )],
    };
    assert!(grouped.is_group_editor());
    assert!(grouped.compact_bar_id().is_some());
    let toggle = Control::Toggle { path: String::new(), value: true };
    assert!(toggle.compact_bar_id().is_none());
    assert!(toggle.is_compact_action());
}

#[test]
fn toggles_use_action_bar() {
    let source = include_str!("../folio.rs");
    assert!(source.contains("fn compact_action_field("));
    assert!(source.contains("crate::frontend::ui::folio_action_bar("));
    assert!(source.contains("crate::frontend::ui::folio_action("));
}

#[test]
fn folio_no_ordinals() {
    let source = include_str!("../folio.rs");
    assert!(!source.contains("format!(\"{:02}\", index + 1)"));
}

#[test]
fn railless_sheet() {
    let source = include_str!("../folio.rs");
    for shared in [
        "crate::frontend::ui::folio_sheet_panel_style(",
        "crate::frontend::ui::folio_scrim_style(",
        "folio_horizontal_rule(",
    ] {
        assert!(source.contains(shared), "{shared}");
    }
    for retired in [
        "folio_masthead(",
        "folio_masthead_with_back(",
        "settings-nav-crumb-root",
        "settings-nav-footer-hint",
        "settings-studio-controls",
    ] {
        assert!(!source.contains(retired), "{retired} came back");
    }
}

#[test]
fn studio_back_and_close() {
    let source = include_str!("../folio.rs");
    assert!(source.contains("tr(\"settings-studio-back\")"));
    assert!(source.contains("SettingsMsg::LeaveLayoutStudio"));
    assert!(source.contains("Message::ToggleSettings"));
}

#[test]
fn workbench_ctx_structs() {
    let source = include_str!("../folio.rs");
    for shaped in [
        "pub struct SourceCtx<",
        "pub struct FocusCtx<",
        "pub struct ChromeCtx<",
        "fn picker_layout_workbench<'a>(\n    source: &SourceCtx<'a>,",
        "pub fn settings_workbench<'a>(input: WorkbenchInput<'a>)",
    ] {
        assert!(source.contains(shaped), "{shaped}");
    }
}

/// R1: one row style, one column; the old two-column grids and the left
/// index are gone; R1b: transition previews play inside cards.
#[test]
fn r1_one_column_rows_and_top_bar() {
    let folio = include_str!("../folio.rs");
    let rows = include_str!("../rows.rs");
    let shell = include_str!("../shell.rs");
    for retired in
        ["fn compact_field_grid", "fn regular_field_grid", "fn navigation<", "fn pinned_preview("]
    {
        assert!(!folio.contains(retired), "{retired} came back");
    }
    assert!(folio.contains("super::rows::rows("));
    assert!(folio.contains("super::shell::top_bar("));
    assert!(rows.contains("SettingsMsg::ResetPath(path)"));
    assert!(!rows.contains("Bubble"), "the hover bubble came back");
    assert!(rows.contains("super::cards::tile_grid("));
    assert!(shell.contains("SettingsMsg::MenuHover(index)"));
    assert!(shell.contains("SettingsMsg::MenuLeave"));
    assert!(shell.contains("SettingsMsg::PaletteChange("));
}

#[test]
fn segment_of_finds_the_heading_above() {
    let row =
        |title: &str, control: Control| Row { title: title.into(), desc: String::new(), control };
    let rows = vec![
        row("a", Control::Static),
        row("Library", Control::Segment),
        row("b", Control::Static),
        row("Combine two", Control::Segment),
        row("c", Control::Static),
    ];
    assert_eq!(segment_of(&rows, 0), None);
    assert_eq!(segment_of(&rows, 2).as_deref(), Some("Library"));
    assert_eq!(segment_of(&rows, 4).as_deref(), Some("Combine two"));
}
