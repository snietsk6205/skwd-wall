mod action;
pub mod background;
mod builder;
mod folio;
mod input_ids;
mod layout;
mod model;
pub mod nav;
mod search;
mod tables;
mod tabs;
#[cfg(test)]
pub(crate) mod test_source;

pub use action::{
    ActionId, SettingsFocus, SettingsKey, SettingsMsg, WEATHER_REVIEW_TAB_COUNT, WeatherReviewTab,
};
pub use folio::READING_PANE;
pub use folio::{
    Backdrop, BigPreview, BubbleView, NavView, PaletteView, RecentItem, card_columns, card_height,
    wide_page_width,
};
pub use folio::{
    ChromeCtx, FocusCtx, FoldersCtx, KeybindCaptureView, ProcessPickerView, ProfilesCtx, SourceCtx,
    WorkbenchInput, picker_layout_workbench, settings_workbench, state_position,
};
pub use folio::{WeatherReviewItem, WeatherReviewView};
pub use input_ids::{
    PRESET_NAME_KEY, palette_input_id, settings_search_input_id, weather_review_input_id,
    workbench_input_id,
};
pub use layout::{
    STUDIO_EDGE_GAP, picker_layout_studio_align, picker_layout_studio_height,
    picker_layout_studio_inset, picker_layout_studio_on_right, picker_layout_studio_width,
    sheet_window,
};
pub use model::{
    Card, Control, FolderLedger, FolderRow, ProfileEntry, ProfilesPage, Row, TagTone, Tile,
    TreeNode, mark_folds,
};
pub use search::{SettingsSearchResult, search_settings};
pub(crate) use tables::SHADERS;
#[cfg(test)]
pub(crate) use tables::is_transition_preview_section;
pub use tables::{
    PageSection, SHADER_FAMILY_KEY, canonical_category, family_default, is_picker_layout_section,
    page_section, visible_tabs,
};
pub(crate) use tabs::RuntimeStatus;
pub use tabs::build_tab;
pub(crate) use tabs::build_tab_with_runtime_status;
pub use tabs::{
    FOLDER_SEGMENT, SEGMENT_PREFIX, STATES, WORKSHOP_TAG, flash_key, folder_ledger,
    profile_commands, profiles_page, segment_key,
};
pub(crate) use tabs::{
    combo_part_family, list_moved, list_toggled, transition_combo_name, transition_list,
};

use builder::Builder;

pub(crate) use tables::NIRI_SNIPPET;
