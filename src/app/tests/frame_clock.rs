use super::*;

#[test]
fn first_surface_seeds_dimensions() {
    let mut app = test_app();
    let id = iced::window::Id::unique();
    let _ = update(&mut app, Message::WindowOpened { id, width: 2560.0, height: 1440.0 });
    assert_eq!(app.scene.viewport, (2560.0, 1440.0));
    assert_eq!(app.runtime_state.overlay, Some(id));
}

#[test]
fn frame_before_surface_noop() {
    let mut app = test_app();
    assert_eq!(app.scene.viewport, (0.0, 0.0));
    assert!(app.runtime_state.last_tick.is_none());

    let _ = update(
        &mut app,
        Message::Daemon(crate::infrastructure::runtime::Wake::Frame(Instant::now())),
    );

    assert!(app.runtime_state.last_tick.is_none());
}

#[test]
fn zero_viewport_suspends_clock() {
    let mut app = test_app();
    app.runtime_state.next_animation_tick = Some(Instant::now());
    app.runtime_state.animation_interval = Some(Duration::from_millis(16));
    app.preview_resources.render_loop_active = true;

    app.schedule_frame();

    assert!(app.runtime_state.next_animation_tick.is_none());
    assert!(app.runtime_state.animation_interval.is_none());
    assert!(!app.preview_resources.render_loop_active);
}

#[test]
fn delayed_frame_uses_consumption_time() {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    let sent_at = Instant::now().checked_sub(Duration::from_secs(2)).unwrap();
    app.input.last_activity = sent_at;
    app.theme.fade_t.run(0.0, 1.0);

    let _ = update(&mut app, Message::Daemon(crate::infrastructure::runtime::Wake::Frame(sent_at)));

    let consumed_at = app.runtime_state.last_tick.expect("frame ticked");
    assert!(consumed_at > sent_at + Duration::from_secs(1));
    let next = app.runtime_state.next_animation_tick.expect("next phase armed");
    assert!(next > consumed_at);
}

#[test]
fn retick_arms_without_double_tick() {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 600);
    assert!(!app.animating());
    assert!(!app.preview_resources.render_loop_active);

    app.input.last_activity = now.checked_sub(Duration::from_secs(1)).unwrap();
    app.theme.fade_t.run(0.0, 1.0);
    app.retick();

    let first = app.runtime_state.last_tick.expect("first tick");
    let next = app.runtime_state.next_animation_tick.expect("next deadline");
    assert!(next > first);

    frame(&mut app, first, 1280.0, 720.0);
    assert_eq!(app.runtime_state.last_tick, Some(first));

    frame(&mut app, next, 1280.0, 720.0);
    assert_eq!(app.runtime_state.last_tick, Some(next));
}

#[test]
fn max_fps_rephases_deadline() {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    app.config.set_key(skwd_config::keys::general::MAX_FPS, json!(20.0));
    app.theme.fade_t.run(0.0, 1.0);
    let last_tick = Instant::now();
    app.input.last_activity = last_tick;
    let slow = app.tick_interval();
    app.runtime_state.last_tick = Some(last_tick);
    app.runtime_state.next_animation_tick = Some(last_tick + slow);
    app.runtime_state.animation_interval = Some(slow);

    app.config.set_key(skwd_config::keys::general::MAX_FPS, json!(120.0));
    let fast = app.tick_interval();
    assert!(fast < slow);
    let scheduled_at = Instant::now();
    app.schedule_frame();
    let accelerated = app.runtime_state.next_animation_tick.expect("active cadence is scheduled");
    assert!(accelerated >= scheduled_at);
    assert!(accelerated <= Instant::now() + fast);

    app.config.set_key(skwd_config::keys::general::MAX_FPS, json!(20.0));
    let slowed_at = Instant::now();
    app.schedule_frame();
    let slowed = app.runtime_state.next_animation_tick.expect("slower cadence is scheduled");
    assert!(slowed >= slowed_at);
    assert!(slowed <= Instant::now() + slow);
}

#[test]
fn idle_schedule_clears_phase() {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 600);
    assert!(!app.animating());

    app.runtime_state.next_animation_tick = Some(now + Duration::from_secs(1));
    app.runtime_state.animation_interval = Some(Duration::from_millis(16));
    app.preview_resources.render_loop_active = true;
    app.schedule_frame();

    assert!(app.runtime_state.next_animation_tick.is_none());
    assert!(app.runtime_state.animation_interval.is_none());
    assert!(!app.preview_resources.render_loop_active);
}

#[test]
fn preview_retargets_and_ignores_stale() {
    let mut app = test_app();
    app.config.set_key(skwd_config::keys::selector::LIVE_PREVIEW, json!(false));
    seed(&mut app, &[wall("first.mp4", "video", 1, 0), wall("second.mp4", "video", 2, 0)]);
    app.scene.viewport = (1280.0, 720.0);
    app.scene.set_preview_config(true, 250, 30);
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 600);
    settle_preview(&mut app);
    assert!(!app.animating());

    app.scene.hover = Some(0);
    app.scene.touch();
    app.retick();
    settle_preview(&mut app);
    let first = app.scene.preview_deadline().expect("preview deadline");
    assert!(!app.animating());
    let before = app.runtime_state.last_tick;
    frame(&mut app, first.checked_sub(Duration::from_millis(1)).unwrap(), 1280.0, 720.0);
    assert_eq!(app.runtime_state.last_tick, before);

    app.scene.hover = Some(1);
    app.scene.touch();
    app.retick();
    settle_preview(&mut app);
    let second = app.scene.preview_deadline().expect("retarget replaces the deadline");
    assert!(second > first);
    let retargeted_at = app.runtime_state.last_tick;
    frame(&mut app, first, 1280.0, 720.0);
    assert_eq!(app.runtime_state.last_tick, retargeted_at);

    app.scene.hover = None;
    app.scene.touch();
    app.retick();
    settle_preview(&mut app);
    assert!(app.scene.preview_deadline().is_none());
    let cancelled_at = app.runtime_state.last_tick;
    frame(&mut app, second, 1280.0, 720.0);
    assert_eq!(app.runtime_state.last_tick, cancelled_at);

    app.scene.hover = Some(0);
    app.scene.touch();
    app.retick();
    settle_preview(&mut app);
    let active = app.scene.preview_deadline().expect("fresh deadline");
    frame(&mut app, active, 1280.0, 720.0);
    assert!(app.runtime_state.last_tick.is_some_and(|last| last >= active));
    assert!(app.scene.preview_deadline().is_none());
}

#[test]
fn settled_hover_stops_frame_loop() {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 600);
    assert!(!app.animating());

    let mut browser = Browser::new(Source::Wallhaven);
    let mut settled = crate::frontend::animation::Spring::for_duration_ms(0.0, 140.0);
    settled.snap(1.0);
    browser.view.hover_fades.insert(0, settled);
    app.source_browser.browser = Some(browser);

    assert!(!app.animating());
    app.source_browser
        .browser
        .as_mut()
        .unwrap()
        .view
        .hover_fades
        .get_mut(&0)
        .unwrap()
        .retarget(0.0);
    assert!(app.animating());
}

#[test]
fn browser_hover_fade_releases() {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 600);
    assert!(!app.animating());

    app.source_browser.browser = Some(Browser::new(Source::Wallhaven));
    std::sync::Arc::make_mut(&mut app.source_browser.wall.scene.render).hits.push(
        crate::frontend::scene::layout::Hit {
            index: 3,
            cx: 100.0,
            cy: 100.0,
            hw: 50.0,
            hh: 50.0,
            skew: 0.0,
            edge_tilt: 0.0,
            hex: false,
            hex_shape: crate::frontend::scene::layout::HexShape::Hexagon,
            triangle_direction: 0,
            quad: None,
        },
    );

    let _ = update(
        &mut app,
        Message::Browser(crate::frontend::browser::BrowserMsg::WallInput(
            crate::frontend::ui::BrowserWallInput::Pointer(100.0, 100.0),
        )),
    );
    let browser = app.source_browser.browser.as_ref().unwrap();
    assert_eq!(browser.session.hover, Some(3));
    assert!(browser.view.hover_fades.contains_key(&3));
    assert!(app.animating());

    tick_frames(&mut app, &mut now, 600);
    let browser = app.source_browser.browser.as_ref().unwrap();
    let spring = browser.view.hover_fades.get(&3).expect("hover fade spring");
    assert!(spring.settled() && spring.value() > 0.99);
    assert!(!app.animating());

    let _ = update(
        &mut app,
        Message::Browser(crate::frontend::browser::BrowserMsg::WallInput(
            crate::frontend::ui::BrowserWallInput::Pointer(2.0, 2.0),
        )),
    );
    assert!(app.animating());

    tick_frames(&mut app, &mut now, 600);
    let browser = app.source_browser.browser.as_ref().unwrap();
    assert!(browser.session.hover.is_none());
    assert!(browser.view.hover_fades.is_empty());
    assert!(!app.animating());
}

#[test]
fn lone_preview_ticks_slow() {
    let mut app = settled_preview_app();
    let full = app.config.frame_interval();
    assert!(app.wants_transition_preview());
    assert!(app.animating());
    assert_eq!(app.tick_interval(), app.config.transition_preview_interval());
    assert!(app.tick_interval() > full);

    // R1b: the live preview only runs while the big preview (or the one
    // that follows the cursor) is open.
    app.panels.settings.nav.big_preview = None;
    assert!(!app.wants_transition_preview());
    assert_eq!(app.tick_interval(), full);
}

#[test]
fn motion_runs_at_frame_rate() {
    let mut app = settled_preview_app();
    assert_eq!(app.tick_interval(), app.config.transition_preview_interval());

    app.input.last_activity = Instant::now().checked_sub(Duration::from_secs(1)).unwrap();
    app.panels.settings.entrance.run(0.0, 1.0);
    assert_eq!(app.tick_interval(), app.config.frame_interval());
}

#[test]
fn fades_run_at_thirty_fps() {
    let mut app = test_app();
    app.config.set_key(skwd_config::keys::general::MAX_FPS, json!(120.0));
    app.input.last_activity = Instant::now().checked_sub(Duration::from_secs(1)).unwrap();
    app.theme.fade_t.run(0.0, 1.0);

    assert!(app.animating());
    assert_eq!(app.tick_interval(), Duration::from_secs_f32(1.0 / 30.0));
}

#[test]
fn passive_loading_twenty_fps() {
    let mut app = test_app();
    app.config.set_key(skwd_config::keys::general::MAX_FPS, json!(120.0));
    app.scene.viewport = (1280.0, 720.0);
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 600);
    app.input.last_activity = Instant::now().checked_sub(Duration::from_secs(1)).unwrap();
    let mut browser = Browser::new(Source::Wallhaven);
    browser.session.loading = true;
    app.source_browser.browser = Some(browser);

    assert!(app.animating());
    assert_eq!(app.tick_interval(), Duration::from_secs_f32(1.0 / 20.0));
}

#[test]
fn scrolling_keeps_frame_rate() {
    let mut app = test_app();
    app.config.set_key(skwd_config::keys::general::MAX_FPS, json!(120.0));
    app.scene.grid_scroll(1.0);

    assert!(app.animating());
    assert_eq!(app.tick_interval(), app.config.frame_interval());
}

#[test]
fn frame_cap_wins() {
    let mut app = test_app();
    app.config.set_key(skwd_config::keys::general::MAX_FPS, json!(24.0));
    app.panels.settings.open = true;
    app.panels.settings.entrance.run(0.0, 1.0);

    assert_eq!(app.tick_interval(), app.config.frame_interval());
}

#[test]
fn non_design_tab_fades_scene() {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    let mut now = Instant::now();
    let _ = update(&mut app, Message::ToggleSettings);
    let _ = update(&mut app, Message::SetSettingsTab(String::from("picker")));
    tick_frames(&mut app, &mut now, 600);
    assert!(app.panels.settings.open);
    assert_eq!(app.scene.render.vis, 0.0);
}

#[test]
fn tertiary_surface_fades_picker() {
    let mut app = test_app();
    app.scene.viewport = (1280.0, 720.0);
    let mut now = Instant::now();
    tick_frames(&mut app, &mut now, 120);
    assert!(app.scene.render.vis > 0.99);

    let _ = update(&mut app, Message::ToggleHelp);
    now += Duration::from_millis(16);
    frame(&mut app, now, 1280.0, 720.0);
    assert!(app.scene.render.vis > 0.0 && app.scene.render.vis < 1.0);

    let _ = update(&mut app, Message::ToggleHelp);
    tick_frames(&mut app, &mut now, 120);
    assert!(app.scene.render.vis > 0.99);
}

#[test]
fn overlay_drops_hover_tint() {
    use crate::app::overlay::Overlay;
    let mut app = test_app();
    app.theme.preview_target = Some(3);
    assert!(!app.menu_capturing());
    app.input.help_open = true;
    assert!(app.menu_capturing());
    app.update_theme_preview(std::time::Instant::now(), 0.016);
    assert!(app.theme.preview_target.is_none());
    app.close_overlay(Overlay::Help);
}

#[test]
fn dwell_gate_blocks_sweeps() {
    use std::time::{Duration, Instant};
    let mut hover = None;
    let start = Instant::now();
    for step in 0..100u64 {
        let now = start + Duration::from_millis(step * 10);
        assert!(!super::theme_bar::dwell_gate(&mut hover, step as usize, now, 50));
    }
    let rest = start + Duration::from_secs(2);
    assert!(!super::theme_bar::dwell_gate(&mut hover, 7, rest, 50));
    assert!(!super::theme_bar::dwell_gate(&mut hover, 7, rest + Duration::from_millis(49), 50));
    assert!(super::theme_bar::dwell_gate(&mut hover, 7, rest + Duration::from_millis(50), 50));
}
