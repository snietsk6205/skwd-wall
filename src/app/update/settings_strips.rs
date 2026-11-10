//! R1b (custom build): the pictures inside the Settings cards. Works out
//! which strips the open page needs, reads the saved ones, queues the rest
//! for the background renderer (never during the benchmark, in Low-power
//! mode, with Battery saver or while Settings is hidden), and plays the
//! strip of the one card the pointer or the keyboard is on.

use std::time::{Duration, Instant};

use iced::advanced::image::Handle;

use crate::app::App;
use crate::app::state::PlayingStrip;
use crate::frontend::settings::{Control, SettingsFocus};
use crate::infrastructure::preview::{
    STRIP_FRAMES, STRIP_H, STRIP_MIDDLE, STRIP_W, StripEvent, StripJob, StripWorker,
};

/// Time one frame of a playing card stays.
const FRAME_MS: u64 = 85;
/// Rest on the last frame before the strip starts again.
const HOLD_MS: u64 = 520;

/// Whether cards may play (they show a still frame otherwise): not in
/// Low-power mode, with Battery saver, with the window's animations off, or
/// with transition previews turned off.
pub(crate) fn motion_allowed(app: &App) -> bool {
    !super::settings_nav::still(app)
        && !app.config.battery_saver_active()
        && app.config.flag_default_true(skwd_config::keys::transition::PREVIEW)
}

/// Whether the background renderer may work now.
fn rendering_allowed(app: &App) -> bool {
    app.panels.settings.open
        && !app.runtime_state.stashed
        && app.runtime_state.perf.bench.is_none()
        && motion_allowed(app)
}

/// The card that is lit: the one under the pointer, else the one the
/// keyboard is on (row of the page, choice; 0 for a card that is a row).
pub(crate) fn lit_tile(app: &App) -> Option<(usize, usize)> {
    let settings = &app.panels.settings;
    settings.nav.tile_hover.or_else(|| {
        (settings.focus == SettingsFocus::Controls && settings.nav.page.is_some())
            .then(|| (settings.focused_control, settings.focused_choice.unwrap_or(0)))
    })
}

fn signature(app: &App) -> String {
    use skwd_config::keys::transition as keys;
    let nav = &app.panels.settings.nav;
    let text = |path: &str| app.config.str_path(path);
    format!(
        "{:?}|{}|{}|{}|{}|{}|{}|{:?}|{}|{}|{}|{}",
        nav.page,
        text(keys::LIBRARY_SEARCH),
        text(keys::LIBRARY_FILTER),
        text(keys::FAVOURITES),
        text(keys::PLAYLIST),
        text(keys::SHADER),
        super::settings_nav::hides_advanced(app),
        nav.menu,
        text(keys::COMBO_FIRST),
        text(keys::COMBO_SECOND),
        text(keys::COMBO_SPLIT),
        text(keys::COMBO_MODE),
    )
}

/// The renderer job for a strip name: a transition key, a combination
/// name, or "sketch:<style>" (that style drawing, then a plain fade).
fn job(name: &str) -> StripJob {
    match name.strip_prefix(crate::frontend::settings::nav::SKETCH_STRIP_PREFIX) {
        Some(style) => StripJob {
            name: name.to_string(),
            shader: String::from("fade"),
            env: vec![(
                String::from("SKWD_PAPER_SKETCH"),
                format!("style={style};ms=3000;base=1000;videos=1;scenes=1"),
            )],
            duration_ms: 4000,
        },
        None => StripJob {
            name: name.to_string(),
            shader: name.to_string(),
            env: Vec::new(),
            duration_ms: 1000,
        },
    }
}

/// Brings the strips in line with the open page (cheap while nothing
/// changed). Called after every update.
pub(crate) fn sync(app: &mut App) {
    app.panels.settings.nav.ink = crate::frontend::ui::ink_palette(&app.theme.palette);
    if !app.panels.settings.open || app.runtime_state.stashed {
        let strips = &mut app.panels.settings.nav.strips;
        strips.playing = None;
        if let Some(worker) = &strips.worker {
            worker.set_paused(true);
        }
        app.panels.settings.nav.cursor_preview = None;
        app.panels.settings.nav.tile_hover = None;
        return;
    }
    let signature = signature(app);
    if app.panels.settings.nav.strips.signature != signature {
        app.panels.settings.nav.strips.signature = signature;
        adopt_page(app);
    }
    let allowed = rendering_allowed(app);
    if let Some(worker) = &app.panels.settings.nav.strips.worker
        && worker.paused() == allowed
    {
        worker.set_paused(!allowed);
    }
    sync_cursor(app);
    sync_playing(app);
}

/// Reads the page's cards and asks for the strips not seen yet.
fn adopt_page(app: &mut App) {
    let mut tiles: Vec<((usize, usize), String)> = Vec::new();
    for (index, row) in super::settings_nav::page_rows(app).iter().enumerate() {
        match &row.control {
            Control::Tile { tile, .. } if !tile.picture.is_empty() => {
                tiles.push(((index, 0), tile.picture.clone()));
            }
            control => {
                if let Some(gallery) = crate::frontend::settings::nav::gallery(control) {
                    for (option, (key, _)) in gallery.options.iter().enumerate() {
                        if let crate::frontend::settings::nav::CardPicture::Strip(name) =
                            gallery.picture(key, &app.config)
                        {
                            tiles.push(((index, option), name));
                        }
                    }
                }
            }
        }
    }
    let mut wanted: Vec<String> = tiles.iter().map(|(_, name)| name.clone()).collect();
    // The Motion menu shows the transition in use on its Transitions card.
    if app.panels.settings.nav.menu.is_some() {
        wanted.push(menu_picture_name(app));
    }
    app.panels.settings.nav.strips.tiles = tiles;
    if wanted.is_empty() {
        return;
    }
    if app.panels.settings.nav.strips.worker.is_none() {
        let cache = app.config.cache_dir();
        let Some(stock) = crate::infrastructure::preview::stock_pair(&cache) else {
            return;
        };
        app.panels.settings.nav.strips.worker =
            StripWorker::start(&cache, stock, app.runtime_state.wake_tx.clone());
    }
    let strips = &mut app.panels.settings.nav.strips;
    let Some(worker) = &strips.worker else {
        return;
    };
    let mut read = Vec::new();
    let mut render = Vec::new();
    for name in wanted {
        if strips.middles.contains_key(&name) {
            continue;
        }
        if worker.cached(&name) {
            if strips.asked.insert(name.clone()) {
                read.push(name);
            }
        } else if !render.iter().any(|queued: &StripJob| queued.name == name) {
            strips.pending.insert(name.clone());
            render.push(job(&name));
        }
    }
    worker.load_middles(read);
    worker.set_queue(render);
}

/// The strip the Motion menu shows on its Transitions card.
pub(crate) fn menu_picture_name(app: &App) -> String {
    let chosen = app.config.str_path(skwd_config::keys::transition::SHADER);
    let name = super::lifecycle::concrete_preview_shader(&app.config, &chosen);
    if name == "random" { String::from("random-mix") } else { name }
}

/// The strip name of a card of the open page.
fn strip_of(app: &App, tile: (usize, usize)) -> Option<String> {
    app.panels
        .settings
        .nav
        .strips
        .tiles
        .iter()
        .find(|(at, _)| *at == tile)
        .map(|(_, name)| name.clone())
}

/// "Preview style: follows the cursor": the small live preview above the
/// card the pointer is on (transitions and combinations; a sketch style
/// plays in its card).
fn sync_cursor(app: &mut App) {
    use skwd_config::keys::settings_window as keys;
    let wanted = (app.config.str_path(keys::PREVIEW_STYLE) == "cursor"
        && motion_allowed(app)
        && app.panels.settings.nav.big_preview.is_none()
        && app.panels.settings.nav.palette.is_none()
        && app.panels.settings.nav.menu.is_none())
    .then_some(app.panels.settings.nav.tile_hover)
    .flatten()
    .and_then(|tile| strip_of(app, tile).map(|name| (tile, name)))
    .filter(|(_, name)| !name.starts_with(crate::frontend::settings::nav::SKETCH_STRIP_PREFIX));
    let current = app
        .panels
        .settings
        .nav
        .cursor_preview
        .as_ref()
        .map(|cursor| ((cursor.row, cursor.option), cursor.shader.clone()));
    if current == wanted {
        return;
    }
    if let Some(((row, option), shader)) = wanted {
        if app.panels.transition_preview.focus.as_deref() != Some(shader.as_str()) {
            app.panels.transition_preview.stop();
        }
        app.panels.transition_preview.focus = Some(shader.clone());
        app.panels.settings.nav.cursor_preview = Some(crate::app::state::CursorPreview {
            row,
            option,
            title: crate::contracts::settings::transitions::label_of(&shader),
            shader,
            x: app.panels.settings.nav.cursor_x,
            target_x: app.panels.settings.nav.cursor_x,
        });
    } else {
        app.panels.settings.nav.cursor_preview = None;
        if app.panels.settings.nav.big_preview.is_none() {
            app.panels.transition_preview.focus = None;
            app.panels.transition_preview.stop();
        }
    }
    app.retick();
}

/// Starts or stops the one playing card.
fn sync_playing(app: &mut App) {
    let cursor =
        app.panels.settings.nav.cursor_preview.as_ref().map(|cursor| (cursor.row, cursor.option));
    let wanted = if motion_allowed(app) && app.panels.settings.nav.big_preview.is_none() {
        lit_tile(app).filter(|tile| Some(*tile) != cursor).and_then(|tile| strip_of(app, tile))
    } else {
        None
    };
    let strips = &mut app.panels.settings.nav.strips;
    if strips.playing.as_ref().map(|playing| &playing.name) == wanted.as_ref() {
        return;
    }
    strips.playing = wanted.map(|name| {
        if let Some(worker) = &strips.worker
            && worker.cached(&name)
        {
            worker.load_frames(&name);
        }
        PlayingStrip { name, frames: Vec::new(), frame: STRIP_MIDDLE, next_at: Instant::now() }
    });
}

fn handle(pixels: &[u8]) -> Handle {
    Handle::from_rgba(STRIP_W, STRIP_H, pixels.to_vec())
}

/// A strip arrived from the background threads. Returns whether to redraw.
pub(crate) fn event(app: &mut App, event: StripEvent) -> bool {
    let strips = &mut app.panels.settings.nav.strips;
    match event {
        StripEvent::Middle { name, pixels } => {
            strips.middles.insert(name, handle(&pixels));
            app.panels.settings.open
        }
        StripEvent::Frames { name, frames } => {
            let Some(playing) = strips.playing.as_mut().filter(|playing| playing.name == name)
            else {
                return false;
            };
            playing.frames = frames.iter().map(|frame| handle(frame)).collect();
            playing.frame = 0;
            playing.next_at = Instant::now() + Duration::from_millis(FRAME_MS);
            app.retick();
            true
        }
        StripEvent::Made { name } => {
            strips.pending.remove(&name);
            strips.asked.insert(name.clone());
            if let Some(worker) = &strips.worker {
                worker.load_middles(vec![name.clone()]);
                if strips.playing.as_ref().is_some_and(|playing| playing.name == name) {
                    worker.load_frames(&name);
                }
            }
            app.panels.settings.open
        }
        StripEvent::Failed { name } => {
            strips.pending.remove(&name);
            strips.asked.insert(name);
            app.panels.settings.open
        }
    }
}

/// Moves the playing card to its next frame when it is due.
pub(crate) fn tick(app: &mut App, now: Instant) {
    let Some(playing) = app.panels.settings.nav.strips.playing.as_mut() else {
        return;
    };
    if playing.frames.is_empty() || now < playing.next_at {
        return;
    }
    playing.frame = (playing.frame + 1) % playing.frames.len().clamp(1, STRIP_FRAMES);
    let last = playing.frame + 1 == playing.frames.len();
    playing.next_at = now + Duration::from_millis(if last { HOLD_MS } else { FRAME_MS });
}

/// The picture a card shows right now.
pub(crate) fn picture(app: &App, name: &str) -> Option<Handle> {
    let strips = &app.panels.settings.nav.strips;
    if let Some(playing) = strips.playing.as_ref().filter(|playing| playing.name == name)
        && let Some(frame) = playing.frames.get(playing.frame)
    {
        return Some(frame.clone());
    }
    strips.middles.get(name).cloned()
}
