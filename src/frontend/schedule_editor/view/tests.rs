#[test]
fn folio_chrome_present() {
    let source = include_str!("../view.rs");
    assert!(source.contains("folio_masthead"));
    assert!(source.contains("folio_sheet("));
    assert!(source.contains("schedule_section("));
    assert!(source.contains("schedule_scale(viewport, window.scale())"));
    assert!(source.contains("folio_action("));
}

#[test]
fn authoring_controls_visible() {
    let source = include_str!("../view.rs");
    let catalog = include_str!("../../../../locales/en-US/schedule.ftl");
    assert!(source.contains("crate::frontend::ui::field_input("));
    assert!(source.contains("folio_button_style(selected, false, palette, status)"));
    assert!(source.contains("schedule-editor-idle-desc"));
    assert!(catalog.contains("schedule-summary-when-label = When"));
    assert!(catalog.contains("schedule-summary-then-label = Then"));
}

#[test]
fn selected_condition_contrast() {
    let source = include_str!("../view.rs");
    assert!(source.contains("with_alpha(palette.primary_text, 0.72)"));
    assert!(source.contains("with_alpha(palette.primary_text, 0.68)"));
}

#[test]
fn retired_chrome_gone() {
    let source = include_str!("../view.rs");
    for retired in ["ScheduleHud", "workbench_surface", "RULE PRIORITY", "PAGE 01 / 01"] {
        assert!(!source.contains(retired));
    }
}

#[test]
fn single_index_and_surface() {
    let source = include_str!("../view.rs");
    let catalog = include_str!("../../../../locales/en-US/schedule.ftl");
    assert_eq!(source.matches("self.priority_index(scale, palette)").count(), 1);
    assert_eq!(source.matches("self.reading_surface(").count(), 1);
    assert!(catalog.contains("The first matching rule wins."));
    assert!(source.contains("schedule-index-subtitle"));
}

#[test]
fn schedule_state_in_index() {
    let source = include_str!("../view.rs");
    let catalog = include_str!("../../../../locales/en-US/schedule.ftl");
    assert!(source.contains("SchedMsg::SetEnabled(true)"));
    assert!(source.contains("SchedMsg::SetEnabled(false)"));
    assert!(source.contains("schedule-state-disabled-desc"));
    assert!(catalog.contains("Rules stay saved, but none of them will run."));
}

#[test]
fn index_pause_and_context() {
    let source = include_str!("../view.rs");
    let catalog = include_str!("../../../../locales/en-US/schedule.ftl");
    assert!(source.contains("SchedMsg::SetRuleEnabled"));
    for key in [
        "schedule-kind-power",
        "schedule-kind-battery",
        "schedule-kind-output",
        "schedule-kind-output-count",
        "schedule-time-hint",
    ] {
        assert!(source.contains(key));
        assert!(catalog.contains(key));
    }
}

#[test]
fn rule_rows_use_stack_bar() {
    let source = include_str!("../view.rs");
    let folio = include_str!("../../ui/misc/folio.rs");
    assert!(source.contains("folio_stack_bar("));
    assert!(source.contains("SchedMsg::ToggleRuleOptions"));
    assert!(source.contains("schedule-rule-state"));
    assert!(folio.contains("pub fn folio_stack_bar"));
    assert!(folio.contains("if expanded { \"▴\" } else { \"▾\" }"));
}

#[test]
fn condition_builder_plain_language() {
    let source = include_str!("../view.rs");
    let catalog = include_str!("../../../../locales/en-US/schedule.ftl");
    for surface in [
        "Match all of these",
        "Match any of these",
        "Must not match",
        "Add condition or group",
        "Nested group: match all",
        "Nested group: match any",
    ] {
        assert!(catalog.contains(surface));
    }
    for wired in [
        "schedule-node-match-all",
        "schedule-node-match-any",
        "schedule-must-not-match",
        "schedule-add-condition-or-group",
        "schedule-add-group-all",
        "schedule-add-group-any",
        "SchedMsg::AddGroup",
        "SchedMsg::SetNegated",
    ] {
        assert!(source.contains(wired));
    }
    for technical in ["ALL · &&", "ANY · ||", "ROOT", "child expression"] {
        assert!(!source.contains(technical));
    }
}

#[test]
fn unfinished_rule_recovery() {
    let source = include_str!("../view.rs");
    let catalog = include_str!("../../../../locales/en-US/schedule.ftl");
    assert!(catalog.contains("Set up this rule's conditions"));
    assert!(catalog.contains("Start with no conditions"));
    assert!(source.contains("schedule-setup-title"));
    assert!(source.contains("schedule-setup-action"));
    assert!(source.contains("SchedMsg::ResetCondition"));
}

#[test]
fn index_and_document_widths() {
    let source = include_str!("../view.rs");
    assert!(source.contains("const SCHEDULE_INDEX_WIDTH: f32 = 360.0;"));
    assert!(source.contains("const SCHEDULE_DEFINITION_SHARE: f32 = 0.38;"));
    assert!(!source.contains("folio_index_shell("));
}

#[test]
fn presentation_scale() {
    assert!((super::schedule_scale((1600.0, 1000.0), 1.0) - 1.18).abs() < f32::EPSILON);
    assert!((super::schedule_scale((1100.0, 700.0), 1.0) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn schedule_no_ordinals() {
    let source = include_str!("../view.rs");
    let catalog = include_str!("../../../../locales/en-US/schedule.ftl");
    assert!(!source.contains("{:02}"));
    assert!(!source.contains("schedule-priority-label"));
    assert!(!catalog.contains("$number"));
    assert!(!catalog.contains("schedule-priority-label"));
}
