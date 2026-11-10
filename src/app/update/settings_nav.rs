//! R1 (custom build): the redesigned Settings window: top bar menus, pages,
//! back, Home cards, Simple/Advanced, reset to default, the command palette,
//! undo and the preview bubble.

use std::time::{Duration, Instant};

use iced::Task;
use serde_json::{Value, json};
use skwd_config::keys::settings_window as keys;

use crate::app::state::{Bubble, MENU_CLOSE_MS, PaletteState, UNDO_STEPS, UndoEntry, UndoToast};
use crate::app::{App, Message};
use crate::frontend::settings::nav::{
    self, GROUPS, PAGES, PaletteAction, PaletteValue, page_content,
};
use crate::frontend::settings::{Card, Control, Row, SettingsFocus, SettingsKey, SettingsMsg};

/// Home cards per row (keyboard Up/Down jump by this many).
pub(crate) const HOME_COLUMNS: usize = 6;
const PALETTE_LIMIT: usize = 9;

pub(super) fn build_tab_for(app: &App, tab: &str) -> Vec<(Card, Vec<Row>)> {
    build_tab(app, tab)
}

fn build_tab(app: &App, tab: &str) -> Vec<(Card, Vec<Row>)> {
    crate::frontend::settings::build_tab_with_runtime_status(
        tab,
        &app.config,
        &app.daemon.effect_themes,
        &app.library_session.all_folders,
        "",
        app.theme.backends.as_deref().unwrap_or(&[]),
        app.settings_runtime_status(),
    )
}

pub(crate) fn simple_view(app: &App) -> bool {
    app.config.str_path(keys::VIEW) != "advanced"
}

/// Whether the current page hides its advanced rows.
pub(crate) fn hides_advanced(app: &App) -> bool {
    let nav = &app.panels.settings.nav;
    simple_view(app)
        && !nav.show_all
        && nav.page.is_none_or(|page| !nav::ADVANCED_PAGES.contains(&PAGES[page].key))
}

/// Rows of the open page as the window shows them.
pub(crate) fn page_rows(app: &App) -> Vec<Row> {
    let Some(page) = app.panels.settings.nav.page else {
        return Vec::new();
    };
    let mut build = |tab: &str| build_tab(app, tab);
    page_content(&PAGES[page], &mut build).rows(hides_advanced(app))
}

/// Tabs the open page takes rows from.
pub(crate) fn page_tabs(app: &App) -> Vec<&'static str> {
    let Some(page) = app.panels.settings.nav.page else {
        return Vec::new();
    };
    let mut tabs: Vec<&'static str> = Vec::new();
    for reference in PAGES[page].refs {
        if !tabs.contains(&reference.tab()) {
            tabs.push(reference.tab());
        }
    }
    tabs
}

pub(crate) fn home_cards(app: &App) -> Vec<usize> {
    let mut out = Vec::new();
    for key in app.config.str_path(keys::HOME_CARDS).split(',') {
        if let Some(index) = nav::page_index(key.trim())
            && !out.contains(&index)
        {
            out.push(index);
        }
    }
    out
}

fn save_home_cards(app: &mut App, cards: &[usize]) {
    let text = cards.iter().map(|index| PAGES[*index].key).collect::<Vec<_>>().join(",");
    super::settings_policy::save_value(app, keys::HOME_CARDS, &json!(text));
}

fn low_power(app: &App) -> bool {
    app.config.low_power()
}

/// Called when Settings opens.
pub(super) fn on_open(app: &mut App) {
    let nav = &mut app.panels.settings.nav;
    nav.history.clear();
    nav.menu = None;
    nav.palette = None;
    nav.bubble = None;
    nav.edit_cards = false;
    nav.toast = None;
    nav.show_all = false;
    let last = nav::page_index(&app.config.str_path(keys::LAST_PAGE));
    let page = if app.config.str_path(keys::OPEN_ON) == "last" { last } else { None };
    show(app, page, None, false);
    if low_power(app) {
        app.panels.settings.nav.settle();
    }
}

/// Called when Settings closes or hides.
pub(crate) fn on_close(app: &mut App) {
    let last = app.panels.settings.nav.page.map_or("", |page| PAGES[page].key);
    if app.config.str_path(keys::LAST_PAGE) != last {
        app.config.set_key(keys::LAST_PAGE, json!(last));
    }
    let nav = &mut app.panels.settings.nav;
    nav.menu = None;
    nav.palette = None;
    nav.bubble = None;
    nav.toast = None;
    nav.edit_cards = false;
    nav.home.streamer = None;
    nav.home.frame = None;
    app.panels.transition_preview.focus = None;
}

/// Shows Home (`None`) or a page, optionally with row `row` (of the page's
/// full row list) focused. `push` keeps the current place for Back.
pub(super) fn show(app: &mut App, page: Option<usize>, row: Option<usize>, push: bool) {
    super::settings::commit_input_edit(app);
    let previous = app.panels.settings.nav.page;
    if push && previous != page {
        let history = &mut app.panels.settings.nav.history;
        history.push(previous);
        if history.len() > 24 {
            history.remove(0);
        }
    }
    let changed = previous != page;
    {
        let nav = &mut app.panels.settings.nav;
        nav.page = page;
        nav.menu = None;
        nav.menu_close_at = None;
        nav.menu_anim.snap(0.0);
        nav.bubble = None;
        nav.big_preview = None;
        nav.tile_hover = None;
        nav.edit_cards = false;
        if changed {
            nav.show_all = false;
            nav.more.clear();
            nav.page_anim.run(0.0, 1.0);
        }
        nav.bar_item = page.map_or(0, |page| PAGES[page].group);
    }
    app.panels.transition_preview.focus = None;
    app.panels.settings.focused_choice = None;
    app.panels.settings.armed = None;
    app.panels.settings.folder_undo = None;
    app.panels.settings.control_page = 0;
    app.panels.settings.focused_control = 0;
    if let Some(index) = page {
        let mut build = |tab: &str| build_tab(app, tab);
        let content = page_content(&PAGES[index], &mut build);
        let primary = content.primary();
        let full: Vec<Row> = content.rows(false);
        if let Some(primary) = primary {
            app.panels.settings.tab = primary.tab.to_string();
            app.panels.settings.section = primary.section;
        }
        let mut target = row.unwrap_or(0);
        if let Some(row) = row
            && hides_advanced(app)
            && full.get(row).is_some_and(nav::row_is_advanced)
        {
            app.panels.settings.nav.show_all = true;
        }
        if row.is_some() && hides_advanced(app) {
            target = visible_index(&full, row.unwrap_or(0));
        }
        app.panels.settings.focus = SettingsFocus::Controls;
        app.panels.settings.focused_control = target;
        if let Some(Row { control, .. }) = page_rows(app).get(target) {
            match control {
                Control::Details { id, .. } if row.is_some() => {
                    app.panels.settings.expanded_details.insert(id.clone());
                }
                control => {
                    if let (Some(id), true) = (control.compact_bar_id(), row.is_some()) {
                        let motion = app.motion_profile();
                        app.panels.settings.open_bar(id, motion);
                    }
                }
            }
        }
        if row.is_none() {
            app.panels.settings.focus = SettingsFocus::Bar;
        }
    } else {
        // Home shows no old section (so nothing like the layout studio).
        app.panels.settings.tab = String::from("home");
        app.panels.settings.section = 0;
        app.panels.settings.focus = SettingsFocus::Home;
        refresh_recent(app);
        let count = home_cards(app).len() + 1;
        let nav = &mut app.panels.settings.nav;
        nav.card_focus = nav.card_focus.min(count.saturating_sub(1));
    }
    if low_power(app) {
        app.panels.settings.nav.settle();
    }
    app.init_settings_inputs();
    app.chrome.pane_scrolls.remove("settings");
    app.invalidate_settings();
    app.retick();
}

/// Index of full-list row `row` among the rows Simple view keeps.
fn visible_index(full: &[Row], row: usize) -> usize {
    full.iter().take(row).filter(|candidate| !nav::row_is_advanced(candidate)).count()
}

/// Esc / Back: one level up, then close Settings.
pub(super) fn back(app: &mut App) -> Task<Message> {
    let nav = &mut app.panels.settings.nav;
    if nav.edit_cards {
        nav.edit_cards = false;
        app.retick();
        return Task::none();
    }
    if nav.page.is_none() && nav.history.is_empty() {
        return super::panels::toggle_settings(app);
    }
    let target = nav.history.pop().unwrap_or(None);
    show(app, target, None, false);
    Task::none()
}

fn open_menu(app: &mut App, group: usize, sticky: bool, keyboard: bool) {
    if !(1..GROUPS.len()).contains(&group) {
        return;
    }
    let unavailable: Vec<usize> = {
        let mut build = |tab: &str| build_tab(app, tab);
        nav::group_pages(group)
            .filter(|(_, page)| !nav::page_has_cards(page, &mut build))
            .map(|(index, _)| index)
            .collect()
    };
    if app.panels.settings.nav.bubble.as_ref().is_some_and(|bubble| !bubble.pinned) {
        close_bubble(app);
    }
    let nav = &mut app.panels.settings.nav;
    if nav.menu != Some(group) {
        nav.menu_item = 0;
        nav.menu_anim.run(0.0, 1.0);
    }
    nav.menu = Some(group);
    nav.menu_sticky = sticky;
    nav.menu_close_at = None;
    nav.menu_unavailable = unavailable;
    nav.bar_item = group;
    if keyboard {
        app.panels.settings.focus = SettingsFocus::Menu;
    }
    if low_power(app) {
        app.panels.settings.nav.settle();
    }
    app.retick();
}

fn close_menu(app: &mut App) {
    let nav = &mut app.panels.settings.nav;
    nav.menu = None;
    nav.menu_close_at = None;
    nav.menu_anim.snap(0.0);
    if app.panels.settings.focus == SettingsFocus::Menu {
        app.panels.settings.focus = SettingsFocus::Bar;
    }
    app.retick();
}

/// Closes a hover menu once its close delay ran out (called every tick).
pub(crate) fn tick_menu(app: &mut App, now: Instant) {
    if let Some(at) = app.panels.settings.nav.menu_close_at
        && now >= at
    {
        close_menu(app);
    }
    let nav = &mut app.panels.settings.nav;
    if let Some(toast) = &nav.toast
        && now.duration_since(toast.since).as_secs_f32() >= crate::app::state::TOAST_SECONDS
    {
        // Fade out, then drop it.
        if nav.toast_anim.target > 0.5 && !app.config.low_power() {
            nav.toast_anim.retarget(0.0);
        } else if nav.toast_anim.x <= 0.0 || app.config.low_power() {
            nav.toast = None;
            nav.toast_anim.snap(0.0);
        }
    }
}

/// Number of menu columns (grid style).
fn menu_columns(app: &App) -> usize {
    if app.config.str_path(keys::MENU_STYLE) == "grid" { 3 } else { 1 }
}

fn menu_pages(group: usize) -> Vec<usize> {
    nav::group_pages(group).map(|(index, _)| index).collect()
}

/// R1 keys. `None` hands the key to the old Settings handling.
pub(super) fn key(app: &mut App, key: SettingsKey) -> Option<Task<Message>> {
    let before = app.panels.settings.nav.page;
    let task = key_inner(app, key)?;
    Some(with_page_scroll(app, before, task))
}

/// A new page starts at its top (or at the row it was opened on).
fn with_page_scroll(app: &App, before: Option<usize>, task: Task<Message>) -> Task<Message> {
    if app.panels.settings.nav.page != before && app.panels.settings.nav.page.is_some() {
        Task::batch([task, scroll_to_focus(app)])
    } else {
        task
    }
}

fn key_inner(app: &mut App, key: SettingsKey) -> Option<Task<Message>> {
    if matches!(key, SettingsKey::CategoryNext { .. }) && app.panels.settings.nav.palette.is_none()
    {
        return None;
    }
    if key == SettingsKey::Undo {
        return Some(undo(app));
    }
    if app.panels.settings.nav.palette.is_some() {
        return Some(palette_key(app, key));
    }
    // While a text field is being edited, the old handling owns the keys.
    if app.panels.settings.input_edit.is_some() {
        return None;
    }
    if key == SettingsKey::Search {
        return Some(open_palette(app));
    }
    if key == SettingsKey::Cancel && app.panels.settings.nav.big_preview.is_some() {
        close_big_preview(app);
        return Some(Task::none());
    }
    if key == SettingsKey::Cancel {
        if app.panels.settings.nav.bubble.as_ref().is_some_and(|bubble| bubble.pinned) {
            close_bubble(app);
            return Some(Task::none());
        }
        if app.panels.settings.nav.menu.is_some() {
            close_menu(app);
            return Some(Task::none());
        }
        return None;
    }
    match app.panels.settings.focus {
        SettingsFocus::Bar => Some(bar_key(app, key)),
        SettingsFocus::Menu => Some(menu_key(app, key)),
        SettingsFocus::Home => Some(home_key(app, key)),
        SettingsFocus::Controls => match key {
            SettingsKey::Previous | SettingsKey::Next | SettingsKey::Up | SettingsKey::Down
                if on_tile(app) =>
            {
                tile_key(app, key)
            }
            SettingsKey::Up
                if app.panels.settings.focused_choice.is_none()
                    && first_focusable(app) == Some(app.panels.settings.focused_control) =>
            {
                app.panels.settings.focus = SettingsFocus::Bar;
                app.retick();
                Some(Task::none())
            }
            _ => None,
        },
    }
}

fn first_focusable(app: &App) -> Option<usize> {
    page_rows(app).iter().position(|row| row.control.is_focusable())
}

/// Top bar items: 0 Home, 1..=5 groups, 6 view switch, 7 palette.
const BAR_ITEMS: usize = 8;

fn bar_key(app: &mut App, key: SettingsKey) -> Task<Message> {
    app.panels.settings.focus = SettingsFocus::Bar;
    let item = app.panels.settings.nav.bar_item.min(BAR_ITEMS - 1);
    match key {
        SettingsKey::Previous => app.panels.settings.nav.bar_item = item.saturating_sub(1),
        SettingsKey::Next => app.panels.settings.nav.bar_item = (item + 1).min(BAR_ITEMS - 1),
        SettingsKey::Down | SettingsKey::Activate => match item {
            0 => {
                if key == SettingsKey::Activate || app.panels.settings.nav.page.is_some() {
                    show(app, None, None, true);
                } else {
                    app.panels.settings.focus = SettingsFocus::Home;
                }
            }
            6 if key == SettingsKey::Activate => toggle_view(app),
            7 if key == SettingsKey::Activate => return open_palette(app),
            6 | 7 => leave_bar(app),
            group => open_menu(app, group, true, true),
        },
        SettingsKey::FocusNext { .. } => leave_bar(app),
        _ => {}
    }
    app.retick();
    Task::none()
}

fn leave_bar(app: &mut App) {
    if app.panels.settings.nav.page.is_some() {
        if let Some(first) = first_focusable(app) {
            app.panels.settings.focus = SettingsFocus::Controls;
            app.panels.settings.focused_control = first;
            app.panels.settings.focused_choice = None;
        }
    } else {
        app.panels.settings.focus = SettingsFocus::Home;
    }
}

fn menu_key(app: &mut App, key: SettingsKey) -> Task<Message> {
    let Some(group) = app.panels.settings.nav.menu else {
        app.panels.settings.focus = SettingsFocus::Bar;
        return Task::none();
    };
    let pages = menu_pages(group);
    let columns = menu_columns(app);
    let item = app.panels.settings.nav.menu_item.min(pages.len().saturating_sub(1));
    let next = match key {
        SettingsKey::Up if item < columns => {
            close_menu(app);
            return Task::none();
        }
        SettingsKey::Up => item - columns,
        SettingsKey::Down => (item + columns).min(pages.len() - 1),
        SettingsKey::Previous if columns == 1 || item.is_multiple_of(columns) => {
            let group = if group <= 1 { GROUPS.len() - 1 } else { group - 1 };
            open_menu(app, group, true, true);
            return Task::none();
        }
        SettingsKey::Next
            if columns == 1 || item % columns == columns - 1 || item + 1 == pages.len() =>
        {
            let group = if group + 1 >= GROUPS.len() { 1 } else { group + 1 };
            open_menu(app, group, true, true);
            return Task::none();
        }
        SettingsKey::Previous => item - 1,
        SettingsKey::Next => item + 1,
        SettingsKey::Activate => {
            if let Some(&page) = pages.get(item) {
                if app.panels.settings.nav.edit_cards && app.panels.settings.nav.page.is_none() {
                    add_card(app, page);
                    close_menu(app);
                    app.panels.settings.focus = SettingsFocus::Home;
                } else {
                    show(app, Some(page), None, true);
                    leave_bar(app);
                }
            }
            return Task::none();
        }
        SettingsKey::FocusNext { .. } => {
            close_menu(app);
            leave_bar(app);
            return Task::none();
        }
        _ => item,
    };
    app.panels.settings.nav.menu_item = next;
    app.retick();
    Task::none()
}

fn home_key(app: &mut App, key: SettingsKey) -> Task<Message> {
    let cards = home_cards(app);
    // The last slot is the "Recently changed" card (or "Add" in edit mode).
    let count = cards.len() + 1;
    let item = app.panels.settings.nav.card_focus.min(count - 1);
    let editing = app.panels.settings.nav.edit_cards;
    let next = match key {
        SettingsKey::Previous => item.saturating_sub(1),
        SettingsKey::Next => (item + 1).min(count - 1),
        SettingsKey::Up if item < HOME_COLUMNS => {
            app.panels.settings.focus = SettingsFocus::Bar;
            app.panels.settings.nav.bar_item = 0;
            app.retick();
            return Task::none();
        }
        SettingsKey::Up => item - HOME_COLUMNS,
        SettingsKey::Down => (item + HOME_COLUMNS).min(count - 1),
        SettingsKey::Activate => {
            if let Some(&page) = cards.get(item) {
                if !editing {
                    show(app, Some(page), None, true);
                    leave_bar(app);
                }
            } else if editing {
                app.panels.settings.nav.menu_item = 0;
                open_menu(app, 1, true, true);
            } else {
                open_recent(app, 0);
            }
            return Task::none();
        }
        SettingsKey::Reorder { earlier } if editing && item < cards.len() => {
            move_card(app, item, earlier);
            if earlier { item.saturating_sub(1) } else { (item + 1).min(cards.len() - 1) }
        }
        SettingsKey::Delete if editing && item < cards.len() => {
            remove_card(app, item);
            item.min(cards.len().saturating_sub(1))
        }
        SettingsKey::FocusNext { .. } => {
            app.panels.settings.focus = SettingsFocus::Bar;
            item
        }
        _ => item,
    };
    app.panels.settings.nav.card_focus = next;
    app.panels.settings.nav.card_anim.run(0.0, 1.0);
    app.retick();
    Task::none()
}

fn move_card(app: &mut App, index: usize, earlier: bool) {
    let mut cards = home_cards(app);
    let target = if earlier { index.checked_sub(1) } else { Some(index + 1) };
    if let Some(target) = target.filter(|target| *target < cards.len()) {
        cards.swap(index, target);
        save_home_cards(app, &cards);
    }
}

fn remove_card(app: &mut App, index: usize) {
    let mut cards = home_cards(app);
    if index < cards.len() {
        cards.remove(index);
        save_home_cards(app, &cards);
    }
}

fn add_card(app: &mut App, page: usize) {
    let mut cards = home_cards(app);
    if !cards.contains(&page) {
        cards.push(page);
        save_home_cards(app, &cards);
    }
}

fn toggle_view(app: &mut App) {
    let next = if simple_view(app) { "advanced" } else { "simple" };
    super::settings_policy::save_value(app, keys::VIEW, &json!(next));
    app.panels.settings.nav.show_all = false;
    let rows = page_rows(app);
    app.panels.settings.focused_control =
        app.panels.settings.focused_control.min(rows.len().saturating_sub(1));
    app.init_settings_inputs();
    app.retick();
}

/// Builds every old tab at most once (lookups over all pages).
fn memo(app: &App) -> impl FnMut(&str) -> Vec<(Card, Vec<Row>)> + '_ {
    let mut cache: std::collections::HashMap<String, Vec<(Card, Vec<Row>)>> =
        std::collections::HashMap::new();
    move |tab: &str| cache.entry(tab.to_string()).or_insert_with(|| build_tab(app, tab)).clone()
}

/// Rebuilds the "Recently changed" lines Home shows.
pub(crate) fn refresh_recent(app: &mut App) {
    let paths = recent_paths(app);
    if paths.is_empty() {
        app.panels.settings.nav.recent.clear();
        return;
    }
    ensure_labels(app);
    let items: Vec<crate::frontend::settings::RecentItem> = paths
        .iter()
        .take(4)
        .filter_map(|path| {
            let label = cached_label(app, path)?;
            let value = app.config.value_at(path).or_else(|| default_value(path))?;
            Some(crate::frontend::settings::RecentItem {
                title: label.title.clone(),
                value: value_text(app, path, &value),
                page: label.page,
                row: label.row,
            })
        })
        .collect();
    app.panels.settings.nav.recent = items;
}

/// Paths of recently changed settings, newest first.
pub(crate) fn recent_paths(app: &App) -> Vec<String> {
    app.config
        .str_path(keys::RECENT)
        .split(',')
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(str::to_string)
        .collect()
}

fn open_recent(app: &mut App, index: usize) {
    let Some(path) = recent_paths(app).get(index).cloned() else {
        return;
    };
    if let Some((page, row)) = find_path(app, &path) {
        show(app, Some(page), Some(row), true);
    }
}

/// Page and row (full list) of the row that edits `path`.
pub(crate) fn find_path(app: &App, path: &str) -> Option<(usize, usize)> {
    if let Some(label) = cached_label(app, path) {
        return Some((label.page, label.row));
    }
    let mut build = memo(app);
    for (index, page) in PAGES.iter().enumerate() {
        let rows = page_content(page, &mut build).rows(false);
        for (row_index, row) in rows.iter().enumerate() {
            if row_edits(row, path) {
                return Some((index, row_index));
            }
        }
    }
    None
}

fn row_edits(row: &Row, path: &str) -> bool {
    match &row.control {
        Control::Toggle { path: own, .. }
        | Control::Number { path: own, .. }
        | Control::TextField { path: own, .. }
        | Control::Resolution { path: own, .. }
        | Control::KeyBinding { path: own, .. }
        | Control::Dropdown { path: own, .. }
        | Control::Chips { path: own, .. } => own == path,
        Control::MotionWeights { weights } => weights.iter().any(|(_, own, _)| own == path),
        Control::Details { rows, .. } | Control::StackBar { rows, .. } => {
            rows.iter().any(|row| row_edits(row, path))
        }
        _ => false,
    }
}

// ---------- values, reset and undo ----------

/// Default value of `path` from the skwd-config schema.
pub(crate) fn default_value(path: &str) -> Option<Value> {
    use skwd_config::schema;
    if let Some(value) = schema::boolean_default(path) {
        return Some(json!(value));
    }
    if let Some(value) = schema::number_default(path) {
        return Some(json!(value));
    }
    schema::text_default(path).map(|value| json!(value))
}

/// Sets `path` like the matching Settings control would (side effects
/// included).
pub(crate) fn apply_value(app: &mut App, path: &str, value: &Value) -> Task<Message> {
    let task = match value {
        Value::Bool(flag) => super::settings::settings_toggle(app, path, *flag),
        Value::String(text) => super::settings::settings_pick(app, path, text),
        Value::Number(number) => {
            let raw = number.to_string();
            super::settings::settings_input_store(app, path, &raw);
            super::settings::settings_commit(app)
        }
        other => {
            super::settings_policy::save_value(app, path, other);
            Task::none()
        }
    };
    if super::settings_policy::is_keybind(path) {
        app.reload_bindings();
    }
    app.init_settings_inputs();
    app.invalidate_settings();
    app.retick();
    task
}

fn reset_path(app: &mut App, path: &str) -> Task<Message> {
    let Some(value) = default_value(path) else {
        return Task::none();
    };
    apply_value(app, path, &value)
}

/// Paths that change as a side effect of using Settings, not by the user.
fn quiet(path: &str) -> bool {
    matches!(path, keys::RECENT | keys::LAST_PAGE | keys::HOME_PAUSED)
        || path.starts_with("transition.library")
        || path.starts_with("transition.combo") && path.ends_with("Family")
}

/// Remembers a change for undo, the toast and "Recently changed".
pub(crate) fn record(app: &mut App, path: &str, before: Option<Value>, after: &Value) {
    // Only changes made in Settings (not picker commands or the bar), and
    // never the benchmark's temporary ones.
    if !app.panels.settings.open
        || app.runtime_state.perf.bench.is_some()
        || quiet(path)
        || before.as_ref() == Some(after)
    {
        return;
    }
    let now = Instant::now();
    if !app.panels.settings.nav.undoing {
        let undo = &mut app.panels.settings.nav.undo;
        match undo.last_mut() {
            Some(last)
                if last.path == path
                    && now.duration_since(last.at) < Duration::from_millis(1500) =>
            {
                last.after = after.clone();
                last.at = now;
            }
            _ => {
                undo.push(UndoEntry {
                    path: path.to_string(),
                    before,
                    after: after.clone(),
                    at: now,
                });
                if undo.len() > UNDO_STEPS {
                    undo.remove(0);
                }
            }
        }
        ensure_labels(app);
        let text = format!("Changed {} to {}", label_of(app, path), value_text(app, path, after));
        set_toast(app, text, true);
    }
    let mut recent = recent_paths(app);
    recent.retain(|other| other != path);
    recent.insert(0, path.to_string());
    recent.truncate(8);
    app.config.set_key(keys::RECENT, json!(recent.join(",")));
    if app.panels.settings.nav.page.is_none() {
        refresh_recent(app);
    } else {
        app.panels.settings.nav.recent.clear();
    }
}

fn set_toast(app: &mut App, text: String, can_undo: bool) {
    let nav = &mut app.panels.settings.nav;
    if nav.toast.is_none() {
        nav.toast_anim.run(0.0, 1.0);
    }
    nav.toast = Some(UndoToast { text, since: Instant::now(), can_undo });
    if app.config.low_power() {
        app.panels.settings.nav.toast_anim.snap(1.0);
    }
    app.retick();
}

pub(super) fn undo(app: &mut App) -> Task<Message> {
    let Some(entry) = app.panels.settings.nav.undo.pop() else {
        set_toast(app, String::from("Nothing to undo"), false);
        return Task::none();
    };
    let value = entry.before.clone().or_else(|| default_value(&entry.path));
    app.panels.settings.nav.undoing = true;
    let task = if let Some(value) = &value {
        apply_value(app, &entry.path, value)
    } else {
        app.config.remove_key(&entry.path);
        app.config.persist();
        Task::none()
    };
    app.panels.settings.nav.undoing = false;
    ensure_labels(app);
    let shown = value
        .map_or_else(|| String::from("its default"), |value| value_text(app, &entry.path, &value));
    let more = app.panels.settings.nav.undo.len();
    let text = if more > 0 {
        format!("Undone: {} is {} again ({more} more to undo)", label_of(app, &entry.path), shown)
    } else {
        format!("Undone: {} is {} again", label_of(app, &entry.path), shown)
    };
    set_toast(app, text, more > 0);
    task
}

fn collect_labels(
    rows: Vec<Row>,
    page: usize,
    parent: Option<usize>,
    out: &mut std::collections::HashMap<String, crate::app::state::SettingLabel>,
) {
    use crate::app::state::SettingLabel;
    for (index, row) in rows.into_iter().enumerate() {
        let at = parent.unwrap_or(index);
        let base = SettingLabel { title: row.title, page, row: at, ..SettingLabel::default() };
        match row.control {
            Control::Details { rows, .. } | Control::StackBar { rows, .. } => {
                collect_labels(rows, page, Some(at), out);
            }
            Control::Toggle { path, .. }
            | Control::TextField { path, .. }
            | Control::Resolution { path, .. }
            | Control::KeyBinding { path, .. } => {
                out.entry(path).or_insert(base);
            }
            Control::Number { path, unit, .. } => {
                out.entry(path).or_insert(SettingLabel { unit, ..base });
            }
            Control::Chips { path, options, .. } | Control::Dropdown { path, options, .. } => {
                out.entry(path).or_insert(SettingLabel { options, ..base });
            }
            Control::MotionWeights { weights } => {
                for (title, path, _) in weights {
                    out.entry(path).or_insert(SettingLabel { title, unit: "ms", ..base.clone() });
                }
            }
            _ => {}
        }
    }
}

fn ensure_labels(app: &mut App) {
    if app.panels.settings.nav.labels.is_some() {
        return;
    }
    let mut out = std::collections::HashMap::new();
    {
        let mut build = memo(app);
        for (index, page) in PAGES.iter().enumerate() {
            collect_labels(page_content(page, &mut build).rows(false), index, None, &mut out);
        }
    }
    app.panels.settings.nav.labels = Some(out);
}

fn cached_label<'a>(app: &'a App, path: &str) -> Option<&'a crate::app::state::SettingLabel> {
    app.panels.settings.nav.labels.as_ref()?.get(path)
}

/// Human name of the setting at `path`.
pub(crate) fn label_of(app: &App, path: &str) -> String {
    cached_label(app, path).map_or_else(
        || {
            find_row(app, path).map_or_else(
                || path.rsplit('.').next().unwrap_or(path).to_string(),
                |row| row.title,
            )
        },
        |label| label.title.clone(),
    )
}

fn find_row(app: &App, path: &str) -> Option<Row> {
    let mut build = memo(app);
    for page in &PAGES {
        for row in page_content(page, &mut build).rows(false) {
            if let Some(found) = find_in(row, path) {
                return Some(found);
            }
        }
    }
    None
}

fn find_in(row: Row, path: &str) -> Option<Row> {
    if !row_edits(&row, path) {
        return None;
    }
    match row.control {
        Control::Details { rows, .. } | Control::StackBar { rows, .. } => {
            rows.into_iter().find_map(|row| find_in(row, path))
        }
        _ => Some(row),
    }
}

/// `value` of `path` in words ("On", a choice's label, "1.2 x").
pub(crate) fn value_text(app: &App, path: &str, value: &Value) -> String {
    let label = cached_label(app, path);
    match value {
        Value::Bool(true) => String::from("On"),
        Value::Bool(false) => String::from("Off"),
        Value::Number(number) => {
            let number =
                crate::contracts::picker::format_config_number(number.as_f64().unwrap_or(0.0));
            match label.map(|label| label.unit) {
                Some(unit) if !unit.is_empty() => format!("{number} {unit}"),
                _ => number,
            }
        }
        Value::String(text) => label
            .and_then(|label| {
                label.options.iter().find(|(key, _)| key == text).map(|(_, words)| words.clone())
            })
            .unwrap_or_else(|| {
                if text.is_empty() { String::from("empty") } else { format!("\"{text}\"") }
            }),
        other => other.to_string(),
    }
}

// ---------- command palette ----------

pub(super) fn open_palette(app: &mut App) -> Task<Message> {
    super::settings::commit_input_edit(app);
    let mut tasks = Vec::new();
    if !app.panels.settings.open {
        tasks.push(super::panels::toggle_settings(app));
    }
    close_menu(app);
    // The bubble floats above everything; it would cover the palette.
    if app.panels.settings.nav.bubble.is_some() {
        close_bubble(app);
    }
    let entries = {
        let mut build = memo(app);
        let mut rows = |index: usize| page_content(&PAGES[index], &mut build).rows(false);
        let layouts: Vec<(&'static str, &'static str)> = crate::contracts::picker::Mode::ALL
            .iter()
            .map(|mode| (mode.as_key(), crate::i18n::tr(mode_label_key(mode.as_key()))))
            .collect();
        nav::palette_index(&mut rows, &layouts)
    };
    app.panels.settings.nav.palette =
        Some(PaletteState { query: String::new(), selected: 0, entries });
    app.retick();
    tasks.push(iced::widget::operation::focus(crate::frontend::settings::palette_input_id()));
    Task::batch(tasks)
}

fn mode_label_key(mode: &str) -> &'static str {
    match mode {
        "depth" => "settings-selector-mode-depth",
        "wall" => "settings-selector-mode-wall",
        "hex" => "settings-selector-mode-hex",
        "sandy" => "settings-selector-mode-sandy",
        "hand" => "settings-selector-mode-hand",
        "collection" => "settings-selector-mode-collection",
        "coverflow" => "settings-selector-mode-coverflow",
        "showcase" => "settings-selector-mode-showcase",
        _ => "settings-selector-mode-slices",
    }
}

/// The ids of the palette's current results, best first.
pub(crate) fn palette_results(app: &App) -> Vec<usize> {
    let Some(palette) = &app.panels.settings.nav.palette else {
        return Vec::new();
    };
    palette_result_indices(palette, &app.panels.settings.nav.palette_recent, &recent_paths(app))
}

pub(crate) fn palette_result_indices(
    palette: &PaletteState,
    recent: &[String],
    changed: &[String],
) -> Vec<usize> {
    let position = |id: &str| palette.entries.iter().position(|entry| entry.id == id);
    if palette.query.trim().is_empty() {
        let mut out: Vec<usize> = recent.iter().filter_map(|id| position(id)).collect();
        for path in changed {
            if let Some(index) = palette.entries.iter().position(|entry| {
                matches!(&entry.value, PaletteValue::Toggle { path: own, .. } | PaletteValue::Choice { path: own, .. } if own == path)
                    || entry.id.ends_with(&format!(":{path}"))
            }) && !out.contains(&index)
            {
                out.push(index);
            }
        }
        for (index, entry) in palette.entries.iter().enumerate() {
            if out.len() >= PALETTE_LIMIT {
                break;
            }
            if matches!(entry.action, PaletteAction::Page(_)) && !out.contains(&index) {
                out.push(index);
            }
        }
        out.truncate(PALETTE_LIMIT);
        return out;
    }
    let found = nav::palette_search(&palette.entries, &palette.query, PALETTE_LIMIT);
    found
        .into_iter()
        .filter_map(|entry| {
            palette.entries.iter().position(|candidate| std::ptr::eq(candidate, entry))
        })
        .collect()
}

fn palette_key(app: &mut App, key: SettingsKey) -> Task<Message> {
    let count = palette_results(app).len();
    let Some(palette) = app.panels.settings.nav.palette.as_mut() else {
        return Task::none();
    };
    match key {
        SettingsKey::Cancel => {
            app.panels.settings.nav.palette = None;
            app.retick();
            return super::settings::unfocus_input();
        }
        SettingsKey::Up => palette.selected = palette.selected.saturating_sub(1),
        SettingsKey::Down => palette.selected = (palette.selected + 1).min(count.saturating_sub(1)),
        SettingsKey::FocusNext { backwards } => {
            let selected = palette.selected;
            return change_result(app, selected, None, backwards);
        }
        SettingsKey::Activate => {
            let selected = palette.selected;
            return run_result(app, selected);
        }
        _ => {}
    }
    app.retick();
    Task::none()
}

fn palette_update(app: &mut App, msg: SettingsMsg) -> Task<Message> {
    match msg {
        SettingsMsg::PaletteOpen => return open_palette(app),
        SettingsMsg::PaletteClose => {
            app.panels.settings.nav.palette = None;
            app.retick();
            return super::settings::unfocus_input();
        }
        SettingsMsg::PaletteInput(query) => {
            if let Some(palette) = app.panels.settings.nav.palette.as_mut() {
                palette.query = query;
                palette.selected = 0;
            }
        }
        SettingsMsg::PaletteSubmit => {
            let selected =
                app.panels.settings.nav.palette.as_ref().map_or(0, |palette| palette.selected);
            return run_result(app, selected);
        }
        SettingsMsg::PaletteRun(index) => return run_result(app, index),
        SettingsMsg::PaletteHover(index) => {
            if let Some(palette) = app.panels.settings.nav.palette.as_mut() {
                palette.selected = index;
            }
        }
        SettingsMsg::PaletteChange(index, value) => {
            return change_result(app, index, value, false);
        }
        _ => {}
    }
    app.retick();
    Task::none()
}

/// Result `position` (in `palette_results`) changed in place: a toggle
/// flips, a choice moves to the next option (or to `value`).
fn change_result(
    app: &mut App,
    position: usize,
    value: Option<String>,
    backwards: bool,
) -> Task<Message> {
    let results = palette_results(app);
    let Some(&entry) = results.get(position) else {
        return Task::none();
    };
    let Some(palette) = app.panels.settings.nav.palette.as_ref() else {
        return Task::none();
    };
    let (path, new) = match &palette.entries[entry].value {
        PaletteValue::Toggle { path, value: on } => (path.clone(), json!(!on)),
        PaletteValue::Choice { path, options, current } => {
            if options.is_empty() {
                return Task::none();
            }
            let next = value.unwrap_or_else(|| {
                let at = options.iter().position(|(key, _)| key == current).unwrap_or(0);
                let next = if backwards {
                    (at + options.len() - 1) % options.len()
                } else {
                    (at + 1) % options.len()
                };
                options[next].0.clone()
            });
            (path.clone(), json!(next))
        }
        PaletteValue::None => return Task::none(),
    };
    if let Some(palette) = app.panels.settings.nav.palette.as_mut() {
        palette.selected = position;
        match &mut palette.entries[entry].value {
            PaletteValue::Toggle { value, .. } => *value = new.as_bool().unwrap_or(false),
            PaletteValue::Choice { current, .. } => {
                *current = new.as_str().unwrap_or_default().to_string()
            }
            PaletteValue::None => {}
        }
    }
    remember_result(app, entry);
    let task = apply_value(app, &path, &new);
    Task::batch([
        task,
        iced::widget::operation::focus(crate::frontend::settings::palette_input_id()),
    ])
}

fn remember_result(app: &mut App, entry: usize) {
    let Some(palette) = app.panels.settings.nav.palette.as_ref() else {
        return;
    };
    let id = palette.entries[entry].id.clone();
    let recent = &mut app.panels.settings.nav.palette_recent;
    recent.retain(|other| *other != id);
    recent.insert(0, id);
    recent.truncate(5);
}

fn run_result(app: &mut App, position: usize) -> Task<Message> {
    let results = palette_results(app);
    let Some(&entry) = results.get(position) else {
        return Task::none();
    };
    let Some(action) = app
        .panels
        .settings
        .nav
        .palette
        .as_ref()
        .map(|palette| palette.entries[entry].action.clone())
    else {
        return Task::none();
    };
    remember_result(app, entry);
    app.panels.settings.nav.palette = None;
    let unfocus = super::settings::unfocus_input();
    let task = match action {
        PaletteAction::Page(page) => {
            show(app, Some(page), None, true);
            leave_bar(app);
            Task::none()
        }
        PaletteAction::Setting { page, row } => {
            show(app, Some(page), Some(row), true);
            scroll_to_focus(app)
        }
        PaletteAction::Transition(key) => {
            let task =
                super::settings::settings_pick(app, skwd_config::keys::transition::SHADER, key);
            if let Some(index) = nav::page_index("motion/transitions") {
                show(app, Some(index), None, true);
            }
            task
        }
        PaletteAction::Layout(mode) => super::settings::set_view_mode(app, mode),
        PaletteAction::Run(id) => run_action(app, id),
    };
    app.retick();
    Task::batch([unfocus, task])
}

fn run_action(app: &mut App, id: &str) -> Task<Message> {
    let message = match id {
        "effects" => Message::KeyStudio,
        "playlists" => Message::OpenPlaylists,
        "mixer" => Message::ToggleAudioPanel,
        "tag-cloud" => Message::OpenTagCloud,
        "downloads" => Message::ToggleDownloads,
        "random" => Message::ApplyRandom,
        "displays" => Message::KeyEffects,
        "schedule" => Message::Settings(SettingsMsg::Run(
            crate::frontend::settings::ActionId::OpenScheduleEditor,
        )),
        "theme-designer" => Message::Settings(SettingsMsg::Run(
            crate::frontend::settings::ActionId::OpenThemeDesigner,
        )),
        "bench" => {
            app.close_settings();
            return super::bench::start(app);
        }
        "home" => {
            show(app, None, None, true);
            return Task::none();
        }
        "edit-cards" => {
            show(app, None, None, true);
            app.panels.settings.nav.edit_cards = true;
            return Task::none();
        }
        "simple-view" => {
            toggle_view(app);
            return Task::none();
        }
        "undo" => return undo(app),
        "close" => return super::panels::toggle_settings(app),
        _ => return Task::none(),
    };
    if !matches!(message, Message::Settings(_)) {
        app.close_settings();
    }
    super::update_inner(app, message)
}

/// Scrolls the page so the focused row is in view (rows are close to the
/// same height in R1, so its share of the list is a good guess).
pub(crate) fn scroll_to_focus(app: &App) -> Task<Message> {
    let page = page_rows(app);
    if app.panels.settings.focus == SettingsFocus::Controls
        && page.iter().any(|row| matches!(row.control, Control::Tile { .. }))
    {
        // A page with cards: work out roughly where the focused row is.
        let (width, scale) = card_grid_width(app);
        let columns = crate::frontend::settings::card_columns(width, scale);
        let card = crate::frontend::settings::card_height(width, scale);
        let focused = app.panels.settings.focused_control;
        let mut y = 150.0 * scale;
        let mut in_line = 0usize;
        for (index, row) in page.iter().enumerate() {
            let tile = matches!(row.control, Control::Tile { .. });
            if !tile && in_line > 0 {
                y += card;
                in_line = 0;
            }
            if index == focused {
                break;
            }
            if tile {
                in_line += 1;
                if in_line == columns {
                    y += card;
                    in_line = 0;
                }
            } else {
                y += if matches!(row.control, Control::Segment) { 66.0 } else { 58.0 } * scale;
            }
        }
        let (_, panel_height) = app.sheet_window().dims(app.scene.viewport);
        return iced::widget::operation::scroll_to(
            crate::frontend::ui::pane_id(crate::frontend::settings::READING_PANE),
            iced::widget::scrollable::AbsoluteOffset {
                x: 0.0,
                y: (y - panel_height * 0.32).max(0.0),
            },
        );
    }
    let rows = page.len();
    let share = if rows < 2 || app.panels.settings.focus != SettingsFocus::Controls {
        0.0
    } else {
        app.panels.settings.focused_control as f32 / (rows - 1) as f32
    };
    iced::widget::operation::snap_to(
        crate::frontend::ui::pane_id(crate::frontend::settings::READING_PANE),
        iced::widget::scrollable::RelativeOffset { x: 0.0, y: share.clamp(0.0, 1.0) },
    )
}

// ---------- big preview ----------

/// Opens the one live preview, large, playing `shader`.
pub(crate) fn open_big_preview(app: &mut App, shader: String) {
    close_bubble(app);
    let nav = &mut app.panels.settings.nav;
    if nav.big_preview.is_none() {
        nav.big_anim.run(0.0, 1.0);
    }
    nav.big_preview = Some(shader.clone());
    nav.menu = None;
    if app.config.low_power() {
        app.panels.settings.nav.big_anim.snap(1.0);
    }
    if app.panels.transition_preview.focus.as_deref() != Some(shader.as_str()) {
        app.panels.transition_preview.stop();
    }
    app.panels.transition_preview.focus = Some(shader);
    app.retick();
}

pub(crate) fn close_big_preview(app: &mut App) {
    if app.panels.settings.nav.big_preview.take().is_some() {
        app.panels.transition_preview.focus = None;
        app.panels.transition_preview.stop();
        app.retick();
    }
}

// ---------- cards ----------

/// Width of the open page's card grids.
fn card_grid_width(app: &App) -> (f32, f32) {
    let window = app.sheet_window();
    let scale = window.scale();
    let (panel_width, _) = window.dims(app.scene.viewport);
    (crate::frontend::settings::wide_page_width(panel_width, scale), scale)
}

/// Whether the keyboard is on a card.
fn on_tile(app: &App) -> bool {
    page_rows(app)
        .get(app.panels.settings.focused_control)
        .is_some_and(|row| matches!(row.control, Control::Tile { .. }))
}

/// Arrow keys on a card: Left/Right and Up/Down move across the grid.
/// With an action chosen (after Enter), Left/Right walk the actions.
fn tile_key(app: &mut App, key: SettingsKey) -> Option<Task<Message>> {
    let rows = page_rows(app);
    let index = app.panels.settings.focused_control;
    let is_tile =
        |row: usize| rows.get(row).is_some_and(|row| matches!(row.control, Control::Tile { .. }));
    let Control::Tile { items, .. } = &rows.get(index)?.control else {
        return None;
    };
    if let Some(choice) = app.panels.settings.focused_choice {
        let count = items.len().max(1);
        match key {
            SettingsKey::Previous => {
                app.panels.settings.focused_choice = Some((choice + count - 1) % count);
            }
            SettingsKey::Next => app.panels.settings.focused_choice = Some((choice + 1) % count),
            SettingsKey::Up | SettingsKey::Down => app.panels.settings.focused_choice = None,
            _ => return None,
        }
        app.retick();
        return Some(Task::none());
    }
    let start = (0..=index).rev().take_while(|row| is_tile(*row)).last().unwrap_or(index);
    let end = (index..rows.len()).take_while(|row| is_tile(*row)).last().unwrap_or(index) + 1;
    let (width, scale) = card_grid_width(app);
    let columns = crate::frontend::settings::card_columns(width, scale);
    let target = match key {
        SettingsKey::Previous => index.saturating_sub(1).max(start),
        SettingsKey::Next => (index + 1).min(end - 1),
        SettingsKey::Up if index - start >= columns => index - columns,
        SettingsKey::Up => (0..start).rev().find(|row| rows[*row].control.is_focusable())?,
        SettingsKey::Down if index + columns < end => index + columns,
        SettingsKey::Down if (index - start) / columns < (end - 1 - start) / columns => end - 1,
        SettingsKey::Down => (end..rows.len()).find(|row| rows[*row].control.is_focusable())?,
        _ => return None,
    };
    app.panels.settings.focused_control = target;
    app.retick();
    Some(scroll_to_focus(app))
}

// ---------- preview bubble ----------

fn show_bubble(app: &mut App, shader: String, row: usize, pinned: bool) {
    if !app.config.flag_default_true(skwd_config::keys::transition::ENABLED)
        || !app.config.flag_default_true(skwd_config::keys::transition::PREVIEW)
        || app.panels.settings.nav.palette.is_some()
        || app.panels.settings.nav.menu.is_some()
    {
        return;
    }
    let nav = &mut app.panels.settings.nav;
    if nav.bubble.as_ref().is_some_and(|bubble| bubble.pinned && !pinned) {
        return;
    }
    if nav.bubble.is_none() {
        nav.bubble_anim.run(0.0, 1.0);
    }
    nav.bubble = Some(Bubble { shader: shader.clone(), row, pinned });
    if app.config.low_power() {
        app.panels.settings.nav.bubble_anim.snap(1.0);
    }
    if app.panels.transition_preview.focus.as_deref() != Some(shader.as_str()) {
        app.panels.transition_preview.stop();
    }
    app.panels.transition_preview.focus = Some(shader);
    app.retick();
}

pub(crate) fn close_bubble(app: &mut App) {
    app.panels.settings.nav.bubble = None;
    app.panels.transition_preview.focus = None;
    app.panels.transition_preview.stop();
    app.retick();
}

/// Shows the bubble for keyboard focus on `row`, or hides an unpinned one.
pub(crate) fn bubble_for_focus(app: &mut App) {
    if app.panels.settings.focus != SettingsFocus::Controls {
        return;
    }
    let index = app.panels.settings.focused_control;
    let shader = page_rows(app).get(index).and_then(|row| bubble_shader(app, row));
    match shader {
        Some(shader) => show_bubble(app, shader, index, false),
        None if app.panels.settings.nav.bubble.as_ref().is_some_and(|bubble| !bubble.pinned) => {
            close_bubble(app);
        }
        None => {}
    }
}

/// What the bubble plays for `row`, if the row is about transitions.
pub(crate) fn bubble_shader(app: &App, row: &Row) -> Option<String> {
    let target = crate::frontend::settings::nav::bubble_target(
        row,
        &app.config.str_path(skwd_config::keys::transition::PLAYLIST),
    )?;
    if !target.is_empty() {
        return Some(target);
    }
    let chosen = app.config.str_path(skwd_config::keys::transition::SHADER);
    Some(if chosen.is_empty() { String::from("random") } else { chosen })
}

// ---------- messages ----------

pub(super) fn update(app: &mut App, msg: SettingsMsg) -> Task<Message> {
    let before = app.panels.settings.nav.page;
    let task = update_inner(app, msg);
    with_page_scroll(app, before, task)
}

fn update_inner(app: &mut App, msg: SettingsMsg) -> Task<Message> {
    match msg {
        SettingsMsg::OpenHome => show(app, None, None, true),
        SettingsMsg::OpenPage(page) => {
            show(app, Some(page), None, true);
            leave_bar(app);
        }
        SettingsMsg::OpenPageRow(page, row) => {
            show(app, Some(page), Some(row), true);
            return scroll_to_focus(app);
        }
        SettingsMsg::Back => return back(app),
        SettingsMsg::MenuClick(group) => {
            if app.panels.settings.nav.menu == Some(group) && app.panels.settings.nav.menu_sticky {
                close_menu(app);
            } else {
                open_menu(app, group, true, false);
            }
        }
        SettingsMsg::MenuHover(group) => {
            let nav = &app.panels.settings.nav;
            if nav.menu == Some(group) {
                app.panels.settings.nav.menu_close_at = None;
            } else if nav.palette.is_none() {
                open_menu(app, group, false, false);
            }
        }
        SettingsMsg::MenuLeave => {
            let nav = &mut app.panels.settings.nav;
            if nav.menu.is_some() && !nav.menu_sticky {
                nav.menu_close_at = Some(Instant::now() + Duration::from_millis(MENU_CLOSE_MS));
                app.retick();
            }
        }
        SettingsMsg::ToggleView => toggle_view(app),
        SettingsMsg::ShowAllRows => {
            app.panels.settings.nav.show_all = true;
            app.init_settings_inputs();
        }
        SettingsMsg::ResetPath(path) => return reset_path(app, &path),
        SettingsMsg::ToggleMore(id) => {
            let more = &mut app.panels.settings.nav.more;
            if !more.remove(&id) {
                more.insert(id);
            }
        }
        SettingsMsg::Undo => return undo(app),
        SettingsMsg::EditCards(on) => {
            app.panels.settings.nav.edit_cards = on;
            app.panels.settings.focus = SettingsFocus::Home;
        }
        SettingsMsg::CardMove(index, earlier) => move_card(app, index, earlier),
        SettingsMsg::CardRemove(index) => remove_card(app, index),
        SettingsMsg::CardAdd(page) => {
            add_card(app, page);
            close_menu(app);
        }
        SettingsMsg::CardHover(index) => {
            let nav = &mut app.panels.settings.nav;
            if nav.card_hover != index {
                nav.card_hover = index;
                nav.card_anim.run(0.0, 1.0);
                if app.config.low_power() {
                    app.panels.settings.nav.card_anim.snap(1.0);
                }
            }
        }
        SettingsMsg::HomePause => {
            let paused = app.config.flag_default_config(keys::HOME_PAUSED);
            super::settings_policy::save_value(app, keys::HOME_PAUSED, &json!(!paused));
        }
        SettingsMsg::BubbleShow(shader, row) => show_bubble(app, shader, row, false),
        SettingsMsg::BubbleHide(row) => {
            if app
                .panels
                .settings
                .nav
                .bubble
                .as_ref()
                .is_some_and(|bubble| !bubble.pinned && bubble.row == row)
            {
                close_bubble(app);
            }
        }
        SettingsMsg::BubblePin(shader, _) => open_big_preview(app, shader),
        SettingsMsg::BigPreviewClose => close_big_preview(app),
        SettingsMsg::TileHover(row, inside) => {
            let nav = &mut app.panels.settings.nav;
            if inside && nav.tile_hover != Some(row) {
                nav.tile_hover = Some(row);
                nav.tile_anim.run(0.0, 1.0);
                if app.config.low_power() {
                    app.panels.settings.nav.tile_anim.snap(1.0);
                }
            } else if !inside && nav.tile_hover == Some(row) {
                nav.tile_hover = None;
            }
        }
        SettingsMsg::BubbleClose => close_bubble(app),
        other => return palette_update(app, other),
    }
    app.invalidate_settings();
    app.retick();
    Task::none()
}
