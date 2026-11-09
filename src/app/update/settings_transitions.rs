//! Custom build (M3): Settings actions of the transition library, the
//! playlist and the "combine two" card. Lists are comma lists in the config;
//! the daemon reads the same keys.

use iced::Task;
use serde_json::json;
use skwd_config::keys::transition as keys;

use crate::app::{App, Message};
use crate::frontend::settings::ActionId;
use crate::frontend::settings::{list_moved, list_toggled, transition_list};

fn save_list(app: &mut App, path: &str, list: &[String]) {
    super::settings_policy::save_value(app, path, &json!(list.join(",")));
}

fn list(app: &App, path: &str) -> Vec<String> {
    transition_list(&app.config.str_path(path))
}

/// Shows `name` (a transition key, "combo" or a combination name) in the
/// preview bubble, pinned (R1: ▶ Preview pins the bubble bigger).
pub(super) fn preview(app: &mut App, name: Option<String>) {
    let Some(name) = name else {
        super::settings_nav::close_bubble(app);
        return;
    };
    let row = app
        .panels
        .settings
        .nav
        .bubble
        .as_ref()
        .map_or(app.panels.settings.focused_control, |bubble| bubble.row);
    let _ = super::settings_nav::update(
        app,
        crate::frontend::settings::SettingsMsg::BubblePin(name, row),
    );
    app.invalidate_settings();
    app.retick();
}

pub(super) fn run(app: &mut App, id: ActionId) -> Task<Message> {
    match id {
        ActionId::PreviewTransition(name) => preview(app, Some(name)),
        ActionId::PlaylistPreview(index) => {
            let entry = list(app, keys::PLAYLIST).get(index).cloned();
            if entry.is_some() {
                preview(app, entry);
            }
        }
        ActionId::UseTransition(name) => {
            app.panels.transition_preview.focus = None;
            return super::settings::settings_pick(app, keys::SHADER, &name);
        }
        ActionId::FavouriteTransition(key) => {
            let favourites = list_toggled(&list(app, keys::FAVOURITES), key);
            save_list(app, keys::FAVOURITES, &favourites);
        }
        ActionId::PlaylistToggle(entry) => {
            let playlist = list_toggled(&list(app, keys::PLAYLIST), &entry);
            save_list(app, keys::PLAYLIST, &playlist);
        }
        ActionId::PlaylistMove(index, earlier) => {
            let playlist = list_moved(&list(app, keys::PLAYLIST), index, earlier);
            save_list(app, keys::PLAYLIST, &playlist);
        }
        ActionId::PlaylistRemove(index) => {
            let mut playlist = list(app, keys::PLAYLIST);
            if index < playlist.len() {
                playlist.remove(index);
            }
            save_list(app, keys::PLAYLIST, &playlist);
        }
        ActionId::PlaylistClear => save_list(app, keys::PLAYLIST, &[]),
        ActionId::PlaylistAddFavourites => {
            let mut playlist = list(app, keys::PLAYLIST);
            for favourite in list(app, keys::FAVOURITES) {
                if !playlist.contains(&favourite) {
                    playlist.push(favourite);
                }
            }
            save_list(app, keys::PLAYLIST, &playlist);
        }
        ActionId::ComboSwap => {
            let first = app.config.str_path(keys::COMBO_FIRST);
            let second = app.config.str_path(keys::COMBO_SECOND);
            let first = if first.is_empty() { String::from("burn") } else { first };
            let second = if second.is_empty() { String::from("ripple") } else { second };
            super::settings_policy::save_value(app, keys::COMBO_FIRST, &json!(second));
            super::settings_policy::save_value(app, keys::COMBO_SECOND, &json!(first));
        }
        _ => return Task::none(),
    }
    app.invalidate_settings();
    app.retick();
    Task::none()
}
