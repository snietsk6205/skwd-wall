use iced::Task;

use crate::frontend::scene::layout::Mode;

#[allow(clippy::wildcard_imports)]
use super::super::*;
use super::browser::browser_apply_current;
use super::effects::effects_nav;
use super::spatial_nav::{Dir, Rect, neighbour};
use super::update_inner;

/// Card rectangles for direction-aware arrow keys: every card's layout
/// position in Grid mode, the cards on screen in Hex mode.
fn spatial_rects(app: &App) -> Option<Vec<Rect>> {
    match app.scene.mode {
        Mode::Grid => Some(
            app.scene
                .grid_placements(
                    app.library_session.library.catalog(),
                    &app.library_session.filtered,
                    app.scene.gp.total_w(),
                )
                .into_iter()
                .map(Rect::from_place)
                .collect(),
        ),
        Mode::Hex => Some(
            app.scene
                .render
                .hits
                .iter()
                .map(|hit| Rect {
                    index: hit.index,
                    cx: hit.cx,
                    cy: hit.cy,
                    hw: hit.hw,
                    hh: hit.hh,
                })
                .collect(),
        ),
        _ => None,
    }
}

/// Moves to the card that is really in direction `dir`. `None` when this
/// layout keeps index navigation or the neighbour is not known (off screen).
fn spatial_move(app: &mut App, dir: Dir) -> Option<()> {
    let rects = spatial_rects(app)?;
    let next = neighbour(&rects, app.scene.current, dir)?;
    app.scene.kb_nav = true;
    app.scene.set_current(next, app.library_session.filtered.len());
    app.retick();
    Some(())
}

fn masonry(app: &App) -> bool {
    app.scene.mode == Mode::Grid
        && app.scene.gp.layout == crate::frontend::scene::layout::GridLayout::Masonry
}

/// Hex cards are numbered column by column, so Left/Right move a whole column.
fn hex_side_step(app: &mut App, forward: bool) {
    let count = app.library_session.filtered.len();
    let rows = app.scene.hp.rows.max(1);
    let cur = app.scene.current;
    let next = if forward { cur + rows } else { cur.saturating_sub(rows) };
    let next = if forward && next >= count { count.saturating_sub(1) } else { next };
    if next != cur && (forward || cur >= rows) {
        app.scene.kb_nav = true;
        app.scene.set_current(next, count);
        app.retick();
    }
}

fn hand_nav_closes_flip(app: &mut App) -> bool {
    if app.scene.mode == Mode::Hand && app.scene.flip_open() && !app.menu_capturing() {
        app.scene.close_flip();
        return true;
    }
    false
}

pub(super) fn wheel(app: &mut App, amount: f32) -> Task<Message> {
    if !hand_nav_closes_flip(app) && (app.detail_open() || app.menu_capturing()) {
        return Task::none();
    }
    app.scene.close_collection();
    match app.scene.mode {
        Mode::Slices | Mode::Depth | Mode::Sandy | Mode::Hand | Mode::Collection => {
            app.scene.slice_scroll(amount, app.library_session.filtered.len());
        }
        Mode::Grid => app.scene.grid_scroll(-amount),
        Mode::Hex => hex_wheel(app, amount),
    }
    app.retick();
    Task::none()
}

fn hex_wheel(app: &mut App, amount: f32) {
    let count = app.library_session.filtered.len() as i64;
    if count == 0 {
        return;
    }
    let rows = app.scene.hp.rows.max(1) as i64;
    let step = app.scene.hp.scroll_step as i64;
    let total_cols = (count + rows - 1) / rows;
    let col = app.scene.current as i64 / rows;
    let new_col = (col - amount.signum() as i64 * step).clamp(0, total_cols - 1);
    let idx = (new_col * rows + app.scene.hex_row as i64).min(count - 1).max(0);
    app.scene.set_current(idx as usize, app.library_session.filtered.len());
}

pub(super) fn key_prev(app: &mut App) -> Task<Message> {
    if app.source_browser.browser.is_some() {
        return browser_key_nav(app, -1);
    }
    if let Some(task) = effects_nav(app, -1, 0) {
        return task;
    }
    if !hand_nav_closes_flip(app) && (app.menu_capturing() || app.detail_open()) {
        return Task::none();
    }
    if app.scene.mode == Mode::Hex {
        if spatial_move(app, Dir::Left).is_none() {
            hex_side_step(app, false);
        }
        return Task::none();
    }
    if masonry(app) {
        spatial_move(app, Dir::Left);
        return Task::none();
    }
    app.scene.kb_nav = true;
    let cur = app.scene.current;
    if cur > 0 {
        app.scene.set_current(cur - 1, app.library_session.filtered.len());
        app.retick();
    }
    Task::none()
}

pub(super) fn key_next(app: &mut App) -> Task<Message> {
    if app.source_browser.browser.is_some() {
        return browser_key_nav(app, 1);
    }
    if let Some(task) = effects_nav(app, 1, 0) {
        return task;
    }
    if !hand_nav_closes_flip(app) && (app.menu_capturing() || app.detail_open()) {
        return Task::none();
    }
    if app.scene.mode == Mode::Hex {
        if spatial_move(app, Dir::Right).is_none() {
            hex_side_step(app, true);
        }
        return Task::none();
    }
    if masonry(app) {
        spatial_move(app, Dir::Right);
        return Task::none();
    }
    app.scene.kb_nav = true;
    let cur = app.scene.current;
    if cur + 1 < app.library_session.filtered.len() {
        app.scene.set_current(cur + 1, app.library_session.filtered.len());
        app.retick();
    }
    Task::none()
}

fn page_step(scene: &crate::app::scene::SceneCore) -> usize {
    let stride = match scene.mode {
        Mode::Grid => scene.gp.cols.max(1),
        Mode::Hex => scene.hp.rows.max(1),
        _ => 1,
    };
    (scene.render.hits.len() / stride).max(1) * stride
}

pub(super) fn key_page(app: &mut App, backwards: bool) -> Task<Message> {
    if app.source_browser.browser.is_some() {
        let step = page_step(&app.source_browser.wall.scene) as i64;
        return browser_seek(app, if backwards { -step } else { step });
    }
    let step = page_step(&app.scene);
    let current = app.scene.current;
    seek(app, if backwards { current.saturating_sub(step) } else { current + step })
}

pub(super) fn key_edge(app: &mut App, last: bool) -> Task<Message> {
    if let Some(browser) = app.source_browser.browser.as_ref() {
        let count = browser.session.items.len() as i64;
        return browser_seek(app, if last { count } else { -count });
    }
    seek(app, if last { usize::MAX } else { 0 })
}

fn browser_seek(app: &mut App, delta: i64) -> Task<Message> {
    let Some(session) = app.source_browser.browser.as_ref().map(|browser| &browser.session) else {
        return Task::none();
    };
    if session.preview.is_some() || session.items.is_empty() {
        return Task::none();
    }
    let last = session.items.len() as i64 - 1;
    let current = app.source_browser.wall.scene.hover.unwrap_or(0) as i64;
    browser_select(app, (current + delta).clamp(0, last) as usize)
}

fn seek(app: &mut App, target: usize) -> Task<Message> {
    if app.panels.effects.is_some()
        || (!hand_nav_closes_flip(app) && (app.menu_capturing() || app.detail_open()))
    {
        return Task::none();
    }
    let count = app.library_session.filtered.len();
    if count == 0 {
        return Task::none();
    }
    app.scene.kb_nav = true;
    app.scene.set_current(target.min(count - 1), count);
    app.retick();
    Task::none()
}

pub(super) fn random_other(count: usize, current: usize, seed: u64) -> usize {
    if count < 2 {
        return 0;
    }
    let pick = (seed % (count as u64 - 1)) as usize;
    if pick >= current { pick + 1 } else { pick }
}

pub(super) fn apply_random(app: &mut App) -> Task<Message> {
    use std::hash::BuildHasher;
    if app.menu_capturing() || app.detail_open() {
        return Task::none();
    }
    let count = app.library_session.filtered.len();
    if count == 0 {
        return Task::none();
    }
    let seed = std::collections::hash_map::RandomState::new().hash_one(std::time::Instant::now());
    let target = random_other(count, app.scene.current, seed);
    app.scene.kb_nav = true;
    app.scene.set_current(target, count);
    app.retick();
    apply_task(app, target)
}

pub(super) fn apply_current(app: &mut App) -> Task<Message> {
    if app.source_browser.browser.is_some() {
        return browser_apply_current(app);
    }
    if app.panels.effects.as_ref().is_some_and(|eff| {
        eff.mode() == crate::frontend::effects::EffectsMode::Studio && eff.has_effects_page()
    }) {
        return update_inner(app, Message::Effects(crate::frontend::effects::EffectsMsg::Apply));
    }
    if app.menu_capturing() {
        return Task::none();
    }
    apply_task(app, app.scene.current)
}

pub(super) fn key_favourite(app: &mut App) -> Task<Message> {
    if app.menu_capturing() {
        return Task::none();
    }
    toggle_favourite(app, app.scene.flipped().unwrap_or(app.scene.current));
    Task::none()
}

pub(super) fn key_reveal(app: &mut App) -> Task<Message> {
    if app.menu_capturing()
        || app.scene.mode != Mode::Hand
        || app.library_session.filtered.is_empty()
    {
        return Task::none();
    }
    app.scene.hand_reveal_toggle();
    app.retick();
    Task::none()
}

pub(super) fn key_flip(app: &mut App) -> Task<Message> {
    if app.menu_capturing() {
        return Task::none();
    }
    if app.detail_open() || app.scene.flip_open() {
        app.scene.close_flip();
        app.retick();
        return Task::none();
    }
    if app.library_session.filtered.is_empty() {
        return Task::none();
    }
    let idx = app.scene.current;
    match app.scene.mode {
        Mode::Slices | Mode::Depth | Mode::Hand => app.scene.toggle_flip(idx),
        Mode::Sandy => {
            app.scene.sandy_settle_now();
            app.scene.toggle_flip(app.scene.current);
        }
        _ => {
            let rect = app
                .scene
                .render
                .hits
                .iter()
                .find(|hit| hit.index == idx)
                .map(|hit| [hit.cx, hit.cy, hit.hw, hit.hh]);
            if let Some(rect) = rect {
                app.scene.open_detail(idx, rect);
            }
        }
    }
    app.retick();
    Task::none()
}

pub(super) fn key_effects(app: &mut App) -> Task<Message> {
    let Some(idx) = effects_target(app) else {
        return Task::none();
    };
    open_effects(app, idx, crate::frontend::effects::EffectsMode::Displays);
    Task::none()
}

pub(super) fn key_studio(app: &mut App) -> Task<Message> {
    let Some(idx) = effects_target(app) else {
        return Task::none();
    };
    super::picker::open_studio(app, idx)
}

fn effects_target(app: &mut App) -> Option<usize> {
    if app.panels.effects.is_some() {
        app.panels.effects = None;
        app.retick();
        return None;
    }
    if app.menu_capturing() || app.library_session.filtered.is_empty() {
        return None;
    }
    Some(app.scene.flipped().unwrap_or(app.scene.current))
}

pub(super) fn key_up(app: &mut App) -> Task<Message> {
    if app.source_browser.browser.is_some() {
        let cols = app.source_browser.wall.scene.gp.cols.max(1) as i64;
        return browser_key_nav(app, -cols);
    }
    if let Some(task) = effects_nav(app, 0, -1) {
        return task;
    }
    if app.menu_capturing() || app.detail_open() {
        return Task::none();
    }
    if app.scene.mode == Mode::Collection {
        return key_prev(app);
    }
    if app.scene.mode == Mode::Grid {
        spatial_move(app, Dir::Up);
        return Task::none();
    }
    if app.scene.mode == Mode::Hex && spatial_move(app, Dir::Up).is_some() {
        return Task::none();
    }
    if app.scene.mode == Mode::Hex && app.scene.hex_row > 0 {
        app.scene.kb_nav = true;
        let col = app.scene.current / app.scene.hp.rows.max(1);
        let idx = col * app.scene.hp.rows + app.scene.hex_row - 1;
        app.scene.set_current(
            idx.min(app.library_session.filtered.len().saturating_sub(1)),
            app.library_session.filtered.len(),
        );
        app.retick();
    }
    Task::none()
}

pub(super) fn key_down(app: &mut App) -> Task<Message> {
    if app.source_browser.browser.is_some() {
        let cols = app.source_browser.wall.scene.gp.cols.max(1) as i64;
        return browser_key_nav(app, cols);
    }
    if let Some(task) = effects_nav(app, 0, 1) {
        return task;
    }
    if app.menu_capturing() || app.detail_open() {
        return Task::none();
    }
    if app.scene.mode == Mode::Collection {
        return key_next(app);
    }
    if app.scene.mode == Mode::Grid {
        spatial_move(app, Dir::Down);
        return Task::none();
    }
    if app.scene.mode == Mode::Hex && spatial_move(app, Dir::Down).is_some() {
        return Task::none();
    }
    if app.scene.mode == Mode::Hex {
        let rows = app.scene.hp.rows.max(1);
        if app.scene.hex_row + 1 < rows {
            let col = app.scene.current / rows;
            let idx = col * rows + app.scene.hex_row + 1;
            if idx < app.library_session.filtered.len() {
                app.scene.kb_nav = true;
                app.scene.set_current(idx, app.library_session.filtered.len());
                app.retick();
            }
        }
    }
    Task::none()
}
