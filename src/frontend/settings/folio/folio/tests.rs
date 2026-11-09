use super::*;

#[test]
fn tracker_spans_rail() {
    assert_eq!(tracker_position(0, 11), 0.0);
    assert_eq!(tracker_position(10, 11), 1.0);
    assert_eq!(tracker_position(99, 11), 1.0);
}

#[test]
fn branch_reveal_height() {
    assert_eq!(branch_height(8, 1.0, 0.0), 0.0);
    assert!(branch_height(8, 1.0, 0.5) > 100.0);
    assert!(branch_height(8, 1.0, 1.0) > branch_height(8, 1.0, 0.5));
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
fn category_copy_complete() {
    for tab in crate::frontend::settings::tables::TABS {
        assert_ne!(category_note(tab), "Settings for this part of skwd-wall.");
    }
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
    assert!(source.contains("fn compact_field_grid<'a>"));
    assert!(source.contains("control.is_inline_editor()"));
    assert!(source.contains("crate::frontend::ui::folio_inline_bar("));
    assert!(source.contains("crate::frontend::ui::folio_stack_bar("));
    assert!(source.contains("SettingsMsg::ToggleBar(id, index)"));
}

#[test]
fn settings_columns_fit_the_available_width_at_large_ui_scales() {
    for (available, scale, single) in [(618.0, 1.0, true), (459.0, 1.5, true), (1072.0, 1.0, false)]
    {
        let (content, column, single_column) = reading_columns(available, scale);
        assert_eq!(single_column, single);
        assert!((content + 76.0 * scale - available).abs() < 0.001);
        assert!((column * 2.0 + 34.0 * scale - content).abs() < 0.001);
    }
}

#[test]
fn inline_vs_group_editors() {
    let inline = [
        Control::Number { key: "number".into(), path: String::new(), unit: "px" },
        Control::TextField { key: "text".into(), path: String::new(), placeholder: "path" },
        Control::KeyBinding { key: "key".into(), path: String::new(), default: "ctrl+f" },
    ];
    assert!(inline.iter().all(Control::is_inline_editor));
    assert!(inline.iter().all(Control::is_compact_field));
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
    assert!(toggle.is_compact_field());
    assert!(Control::Code { snippet: "layer-rule" }.is_wide_field());
}

#[test]
fn toggles_use_action_bar() {
    let source = include_str!("../folio.rs");
    assert!(source.contains("fn compact_action_field("));
    assert!(source.contains("crate::frontend::ui::folio_action_bar("));
    assert!(source.contains("crate::frontend::ui::folio_action("));
    assert!(source.contains("row.control.is_compact_field() == compact"));
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
        "crate::frontend::ui::folio_index_shell(",
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
fn page_never_covers_reading() {
    let source = include_str!("../folio.rs");
    let page_start = source.find("let page = container(stack![").unwrap();
    let body_start = source[page_start..].find("let body = row![").unwrap() + page_start;
    let page = &source[page_start..body_start];
    assert!(page.contains("reading,"));
    assert!(page.contains(".style(move |_|"));
    assert!(!page.contains("stack![\n        atmosphere,\n        container(text(\"\"))"));
}

#[test]
fn preview_uses_full_stage() {
    let mut settings = vec![
        Row { title: String::new(), desc: String::new(), control: Control::Preview },
        Row { title: String::new(), desc: String::new(), control: Control::Static },
    ];
    assert!(take_transition_preview(&mut settings));
    assert_eq!(settings.len(), 1);
    assert!(matches!(settings[0].control, Control::Static));
    let source = include_str!("../folio.rs");
    // Custom build (M3): the stage is pinned above the scrolling list.
    assert!(source.contains("let stage = pinned_preview("));
    assert!(source.contains("transition_preview_stage(handle, stage_width, scale, palette, fade)"));
    assert!(source.contains("return column![top, list]"));
    assert!(!source.contains("if let Some(handle) = transition_preview {\n        image(handle)"));
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
        "struct ReadingInput<",
        "fn picker_layout_workbench<'a>(\n    source: &SourceCtx<'a>,",
        "fn reading_surface<'a>(input: ReadingInput<'a>, focus: FocusCtx<'_>)",
    ] {
        assert!(source.contains(shaped), "{shaped}");
    }
}
