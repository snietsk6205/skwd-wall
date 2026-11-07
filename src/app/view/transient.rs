use iced::widget::container;
use iced::{Alignment, Element, Length, Padding};

#[allow(clippy::wildcard_imports)]
use super::super::*;

const EMPTY_FOLDER_KEY: &str = "status-library-folder-empty";

pub(crate) fn help_intent_message(intent: crate::frontend::ui::HelpIntent) -> Message {
    match intent {
        crate::frontend::ui::HelpIntent::Close => Message::ToggleHelp,
        crate::frontend::ui::HelpIntent::Capture => Message::Noop,
    }
}

pub(super) fn library_hint(app: &App) -> Option<Element<'_, Message>> {
    if let Some((id, name, _)) = &app.library_session.playlist_filter {
        let title =
            crate::i18n::tr_args!("playlists-picker-entry", name => name, id => id.to_string());
        let width =
            crate::frontend::ui::folio_action_width_in(&title, app.config.ui_scale(), 100.0, 400.0);
        let chip = crate::frontend::ui::folio_action(
            title,
            true,
            Some(Message::Pl(crate::frontend::playlists::PlMsg::OpenPicker)),
            Length::Fixed(width),
            app.config.ui_scale(),
            &app.theme.palette,
        );
        return Some(
            container(chip).width(Length::Fill).align_x(Alignment::Center).padding(16).into(),
        );
    }
    let folder = &app.library_session.filters.folder;
    let empty_source = app.library_session.filtered.is_empty()
        && skwd_config::library_folders(app.config.root()).contains(folder)
        && !app.library_session.library.catalog().items.iter().any(|item| {
            item.kind != crate::domain::library::catalog::WallpaperKind::We
                && std::path::Path::new(&item.path).starts_with(folder)
        });
    let hint = if empty_source && app.daemon.connected {
        crate::i18n::tr_args!(
            EMPTY_FOLDER_KEY,
            directory => folder.as_str()
        )
    } else {
        empty_library_hint(
            app.library_session.library.catalog().items.len(),
            skwd_config::library_destination(app.config.root(), false).as_deref(),
            app.daemon.connected,
            app.daemon.ever_connected,
        )?
    };
    let color = crate::frontend::ui::with_alpha(app.theme.palette.surface_text, 0.75);
    let hint_text = iced::widget::text(hint)
        .size(16)
        .font(crate::frontend::ui::UI_FONT)
        .color(color)
        .align_x(Alignment::Center)
        .width(Length::Fixed(520.0));
    Some(
        container(hint_text)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .padding(24)
            .into(),
    )
}

pub(super) fn help_layer(app: &App) -> Option<Element<'_, Message>> {
    app.input.help_open.then(|| {
        crate::frontend::ui::help_overlay(
            &app.input.bindings,
            &app.theme.palette,
            app.config.ui_scale(),
        )
        .map(help_intent_message)
    })
}

#[cfg(test)]
mod tests;

fn perf_overlay_text(app: &App) -> String {
    use crate::infrastructure::observability::perf;
    let state = &app.runtime_state.perf;
    let (fps, p50, p95) = state.frame_stats();
    let span = |name: &str| perf::stat(name);
    let click = span("update.settings");
    let view = span("view");
    let settings_view = span("view.settings");
    let layout = span("apply_layout");
    let inputs = span("settings.init_inputs");
    let save = span("config.save");
    let tick = span("tick");
    // No recent animation frames to measure (idle, or frames over 100 ms).
    let frames = if fps > 0.0 {
        format!("FPS {fps:>5.1}   frame p50 {p50:>5.1} ms  p95 {p95:>5.1} ms")
    } else {
        String::from("FPS    --   (nothing animating right now)")
    };
    let mut text = format!(
        "{frames}\n\
         CPU {:>5.1} %   redraws {}\n\
         settings click  last {:>6.2}  avg {:>6.2}  max {:>6.2} ms\n\
         settings view   last {:>6.2}  avg {:>6.2}  max {:>6.2} ms\n\
         whole view      last {:>6.2}  avg {:>6.2}  max {:>6.2} ms\n\
         frame tick      last {:>6.2}  avg {:>6.2}  max {:>6.2} ms\n\
         layout {:>5.2}  inputs {:>5.2}  config save {:>5.2} ms (avg)\n\
         low power {}   battery saver {}",
        state.cpu.percent,
        crate::app::view::view_count(),
        click.last_ms(),
        click.mean_ms(),
        click.max_ms(),
        settings_view.last_ms(),
        settings_view.mean_ms(),
        settings_view.max_ms(),
        view.last_ms(),
        view.mean_ms(),
        view.max_ms(),
        tick.last_ms(),
        tick.mean_ms(),
        tick.max_ms(),
        layout.mean_ms(),
        inputs.mean_ms(),
        save.mean_ms(),
        if app.config.low_power() { "on" } else { "off" },
        if app.config.battery_saver_active() { "active" } else { "off" },
    );
    if state.bench.is_some() {
        text.push_str("\nbenchmark running...");
    }
    if app.runtime_state.metrics.hud_enabled {
        text.push_str("\n\n");
        text.push_str(&app.runtime_state.metrics.hud_lines());
    }
    text
}

pub(super) fn hud_layer(app: &App) -> Option<Element<'_, Message>> {
    (app.runtime_state.metrics.hud_enabled || app.config.perf_overlay()).then(|| {
        let hud = crate::contracts::presentation::HudSnapshot {
            metrics: perf_overlay_text(app),
            daemon: if app.runtime_state.metrics.hud_enabled {
                app.daemon.diagnostic.clone()
            } else {
                String::new()
            },
        };
        container(crate::frontend::ui::hud(&hud, app.palette(), app.config.ui_scale()))
            .width(Length::Fill)
            .align_x(Alignment::End)
            .padding(10)
            .into()
    })
}

pub(super) fn toast_layer(app: &App) -> Option<Element<'_, Message>> {
    let (msg, since) = app.runtime_state.toast.as_ref()?;
    let elapsed = app.runtime_state.last_tick.unwrap_or(*since).duration_since(*since).as_millis();
    if elapsed >= TOAST_MS {
        return None;
    }
    let remaining = (TOAST_MS - elapsed) as f32;
    let fade_ms = app.motion_profile().duration_ms(crate::frontend::animation::MotionTier::Slow);
    let alpha = (remaining / fade_ms).min(1.0);
    let pal = app.theme.palette;
    let toast_max = (app.scene.viewport.0 * 0.8).clamp(160.0, 500.0);
    let toast = container(
        iced::widget::text(msg.clone())
            .size(13)
            .color(crate::frontend::ui::with_alpha(pal.primary_text, alpha)),
    )
    .padding(Padding::from([8.0, 14.0]))
    .max_width(toast_max)
    .style(move |_t| {
        crate::frontend::ui::bg_style(crate::frontend::ui::with_alpha(pal.primary, 0.92 * alpha))
    });
    Some(
        container(toast)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::End)
            .padding(Padding { bottom: 40.0, ..Padding::ZERO })
            .into(),
    )
}
