use crate::contracts::settings::keys;

use super::tables::{NIRI_SNIPPET, POST_TYPES, motion_speed_options};
use super::{ActionId, Builder, Card, Control, PRESET_NAME_KEY, Row};

mod app_themes;
mod builder;
mod folders_tab;
pub use folders_tab::{flash_key, folder_ledger};
mod launch_tab;
mod layouts_tab;
mod media_tabs;
mod motion_tab;
mod position_tab;
mod profiles_tab;
pub use profiles_tab::{
    FOLDER_SEGMENT, SEGMENT_PREFIX, STATES, WORKSHOP_TAG, profile_commands, profiles_page,
    segment_key,
};
mod selector_tab;
mod source_defaults;
mod system_tabs;
pub(crate) use system_tabs::weather_category_label;
mod theme_tabs;
mod transition_library;
pub(crate) use transition_library::{
    COMBO_CARD, LIBRARY_CARD, PLAYLIST_CARD, combo_part_family, list_moved, list_toggled,
    transition_combo_name, transition_list,
};
mod transition_tab;
mod window_tab;

pub(crate) use builder::RuntimeStatus;
pub use builder::build_tab;
pub(crate) use builder::build_tab_with_runtime_status;

mod tests;
