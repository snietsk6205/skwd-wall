use std::time::Instant;

use iced::Task;
use log::info;

#[allow(clippy::wildcard_imports)]
use super::super::*;

pub(crate) fn drive_transition_preview(app: &mut App) {
    if !app.wants_transition_preview() {
        app.panels.transition_preview.stop();
        return;
    }
    let shader = preview_shader(app);
    let fill_mode = app.config.str_path(skwd_config::keys::display::FILL_MODE);
    let fill_mode = if fill_mode.is_empty() { String::from("fill") } else { fill_mode };
    let base =
        app.config.num_path(skwd_config::keys::transition::DURATION_MS).clamp(300.0, 10_000.0)
            as u64;
    let sketch_env = preview_sketch_env(&app.config, base);
    let duration = if sketch_env.is_empty() {
        base
    } else {
        (base + preview_sketch_ms(&app.config)).min(10_000)
    };
    let origin = app.config.str_path(skwd_config::keys::transition::ORIGIN);
    let env = [
        ("SKWD_PAPER_SKETCH", sketch_env),
        (
            "SKWD_PAPER_TRANSITION_ORIGIN",
            if origin.is_empty() { String::from("default") } else { origin },
        ),
    ];
    let Some((fallback, to)) = crate::infrastructure::preview::stock_pair(&app.config.cache_dir())
    else {
        return;
    };
    let selected = app.selected_settings_preview_path();
    let from = if !selected.is_empty() && std::path::Path::new(selected).is_file() {
        selected.to_string()
    } else {
        fallback
    };
    let frame_ms = app.config.transition_preview_interval().as_millis() as u64;
    app.panels.transition_preview.ensure(
        &from,
        &to,
        &shader,
        &fill_mode,
        duration,
        frame_ms.max(4),
        &env,
    );
    let first = app.panels.transition_preview.handle.is_none();
    if app.panels.transition_preview.take_frame() && first {
        app.invalidate_settings();
    }
}

/// Custom build (M3): the transition the live preview plays: the one ▶ Preview
/// asked for, else the chosen one; "playlist" previews its first entry and
/// "combo" the configured combination.
pub(crate) fn preview_shader(app: &App) -> String {
    let chosen = app.config.str_path(skwd_config::keys::transition::SHADER);
    let name = app.panels.transition_preview.focus.clone().unwrap_or(chosen);
    concrete_preview_shader(&app.config, &name)
}

pub(crate) fn concrete_preview_shader(
    config: &crate::infrastructure::config::Config,
    name: &str,
) -> String {
    let name = if name == "playlist" {
        crate::frontend::settings::transition_list(
            &config.str_path(skwd_config::keys::transition::PLAYLIST),
        )
        .into_iter()
        .next()
        .unwrap_or_else(|| String::from("random"))
    } else if name.is_empty() {
        String::from("random")
    } else {
        name.to_string()
    };
    if name == "combo" { crate::frontend::settings::transition_combo_name(config) } else { name }
}

fn preview_sketch_ms(config: &crate::infrastructure::config::Config) -> u64 {
    let sketch = config.num_path(skwd_config::keys::transition::SKETCH_DURATION_MS);
    if sketch > 0.0 { sketch.clamp(1000.0, 20_000.0) as u64 } else { 6000 }
}

/// Same `SKWD_PAPER_SKETCH` value the daemon gives its renderers, so the
/// Settings preview shows the sketch exactly as the desktop will.
fn preview_sketch_env(config: &crate::infrastructure::config::Config, base: u64) -> String {
    if !config.flag_default_config(skwd_config::keys::transition::SKETCH_ENABLED) || base >= 10_000
    {
        return String::new();
    }
    let style = config.str_path(skwd_config::keys::transition::SKETCH_STYLE);
    format!(
        "style={};ms={};base={base};videos={};scenes={}",
        if style.is_empty() { "pencil" } else { style.as_str() },
        preview_sketch_ms(config),
        u8::from(config.flag_default_true(skwd_config::keys::transition::SKETCH_VIDEOS)),
        u8::from(config.flag_default_true(skwd_config::keys::transition::SKETCH_SCENES)),
    )
}

pub(super) fn tick(app: &mut App, now: Instant, w: f32, h: f32) -> Task<Message> {
    let animating = app.animating();
    let settings_due = app.panels.settings.open
        && app.panels.settings.nav.deadline().is_some_and(|deadline| now >= deadline);
    let wake_due = animating || app.scene.preview_deadline_due(now) || settings_due;
    if !crate::app::scene::animation_tick_due(wake_due, app.runtime_state.next_animation_tick, now)
    {
        app.preview_resources.render_loop_active = animating;
        return Task::none();
    }
    if (w, h) != app.scene.viewport {
        app.scene.viewport = (w, h);
        if app.config.set_screen_width(w) {
            app.apply_layout();
        }
    }
    app.run_tick(now);
    let mut tasks: Vec<Task<Message>> = Vec::new();
    if app.tags.focus_pending
        && app.tags.editing
        && app.scene.render.back.as_ref().is_some_and(|back| back.add_open > 0.85)
    {
        app.tags.focus_pending = false;
        tasks.push(iced::widget::operation::focus(tag_input_id()));
    }
    for (key, pane) in &mut app.chrome.pane_scrolls {
        if pane.dirty {
            pane.dirty = false;
            tasks.push(iced::widget::operation::scroll_to(
                crate::frontend::ui::pane_id(key),
                iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: pane.spring.x },
            ));
        }
    }
    Task::batch(tasks)
}

pub(super) fn exit(app: &mut App) -> Task<Message> {
    if app.theme.audition_open && app.theme.audition_focused {
        super::warm::exit_picker(app);
    }
    if app.close_topmost_overlay() {
        return Task::none();
    }
    if app.scene.close_collection() {
        app.retick();
        return Task::none();
    }
    app.runtime_state.metrics.exit_summary();
    info!("close requested, rss = {}", crate::infrastructure::observability::rss_label());
    super::warm::exit_picker(app)
}
