use iced::Task;

#[allow(clippy::wildcard_imports)]
use super::*;

pub(super) fn finish(
    app: &mut App,
    task: Task<Message>,
    detail_was_open: bool,
    sync_settings_preview: bool,
    sync_filter_bar: bool,
) -> Task<Message> {
    if !app.panels.audio_volumes.is_empty()
        && app.panels.audio.is_none()
        && app.panels.effects.is_none()
        && !(app.panels.settings.open && app.panels.settings.tab == "displays")
    {
        app.panels.audio_volumes.clear();
        app.call_tracked("wall.outputs", serde_json::json!({}), Pending::Outputs);
    }
    if detail_was_open != app.detail_open() {
        app.chrome.bar.cache.clear();
    }
    app.sync_filter_bar_chrome();
    let hard_overlay = app.picker_obscured();
    let (design_mode, scene_hidden) = scene_visibility(
        app.panels.settings.open,
        &app.panels.settings.tab,
        app.panels.settings.section,
        hard_overlay,
    );
    let demo_hidden =
        app.runtime_state.demo.as_ref().is_some_and(|session| session.picker_suppressed);
    app.scene.set_visible(!scene_hidden && !demo_hidden);
    let rtl = crate::i18n::is_rtl();
    let window = app.sheet_window();
    let (inset, inset_on_right) = if design_mode {
        (
            crate::frontend::settings::picker_layout_studio_inset(app.scene.viewport.0, &window),
            crate::frontend::settings::picker_layout_studio_on_right(&window, rtl),
        )
    } else if app.panels.playlists.is_some() && !scene_hidden {
        (PLAYLIST_DOCK_W, rtl)
    } else {
        (0.0, rtl)
    };
    app.panels.settings.inset_target = if design_mode { inset } else { 0.0 };
    app.scene.set_reading_direction(rtl);
    if inset > 0.0 {
        // Keep the old side while the picker slides back, so it does not jump.
        app.scene.set_inset_side(inset_on_right);
    }
    app.scene.set_center_inset(inset);
    if sync_filter_bar {
        let bar_footprint = (app.chrome.filter_bar_fade() > 0.001 && !hard_overlay).then(|| {
            let (vw, vh) = app.scene.viewport;
            crate::app::view::filter_bar_footprint(app, vw, vh)
        });
        app.scene.set_filter_bar_footprint(bar_footprint);
    }
    app.schedule_frame();
    if sync_settings_preview { Task::batch([task, app.sync_settings_preview()]) } else { task }
}
