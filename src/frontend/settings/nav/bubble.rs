//! R1: which rows show the floating transition preview, and what it plays.

use super::super::{ActionId, Control, Row};
use crate::contracts::settings::keys::transition as keys;

/// What the preview bubble plays for `row`: the transition of a library
/// row, a playlist entry, the combination, or the chosen transition for the
/// transition and sketch choices. `playlist` is the `transition.playlist`
/// comma list.
pub fn bubble_target(row: &Row, playlist: &str) -> Option<String> {
    match &row.control {
        Control::ActionChips { items } => items.iter().find_map(|(id, _)| match id {
            ActionId::PreviewTransition(name) => Some(name.clone()),
            ActionId::PlaylistPreview(index) => {
                super::super::transition_list(playlist).get(*index).cloned()
            }
            _ => None,
        }),
        Control::Chips { path, .. } | Control::Dropdown { path, .. } => match path.as_str() {
            keys::SHADER | keys::SKETCH_STYLE | keys::ORIGIN => Some(String::new()),
            path if path.starts_with("transition.combo") => Some(String::from("combo")),
            _ => None,
        },
        Control::Toggle { path, .. } if path == keys::SKETCH_ENABLED => Some(String::new()),
        _ => None,
    }
}
