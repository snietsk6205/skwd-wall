//! R1: which rows the Simple view hides. Search and the palette still find
//! them, opening one from there shows it, and every page has "Show them".
//! The list (and why) is in handoff/HANDOFF.md, Phase 10.

use super::super::{Control, Row};
use crate::contracts::settings::keys::{
    general, library, matugen, motion, niri, paper, performance, playback, selector, semantic,
    sources, system, theme, transition, we_render, weather,
};

/// Pages where every row counts as advanced. In Simple view their menu
/// entries say "Advanced"; opening one still shows everything.
pub const ADVANCED_PAGES: [&str; 5] =
    ["motion/awww", "motion/quality", "playback/report", "wallpapers/library", "system/after"];

/// Config paths of advanced rows: fine geometry, engine internals, timing
/// knobs and expert colour options.
const ADVANCED: &[&str] = &[
    // Fine geometry of the layouts (Positions, Picker motion).
    selector::GRID_STAGE_SHEAR_X,
    selector::GRID_STAGE_DEPTH_ANGLE,
    selector::HEX_STAGE_SHEAR_X,
    selector::HEX_STAGE_DEPTH_ANGLE,
    selector::SKEW_OFFSET,
    selector::SLICE_EDGE_TILT,
    selector::SLICE_WOBBLE_STRENGTH,
    selector::DEPTH_FALLOFF_FACTOR,
    selector::DEPTH_SKEW,
    selector::SANDY_SKEW,
    selector::SANDY_WAIST,
    selector::SANDY_TWIST,
    selector::SANDY_TURBULENCE,
    selector::SANDY_LOD,
    selector::SANDY_LOD_AUTO,
    selector::SANDY_RES_SCALE,
    selector::SANDY_STRANDS,
    selector::SANDY_EDGE_SPEED,
    selector::SANDY_RING_SOFT,
    selector::SANDY_RING_WAVE,
    selector::SANDY_RING_HOLD,
    selector::SANDY_RING_BLEND,
    selector::SANDY_SWAP_LOOP,
    selector::SANDY_OUTGOING_LIVE,
    selector::HAND_SKEW,
    selector::HAND_PERSPECTIVE,
    selector::HAND_CUT_VARIANCE,
    selector::HAND_FAN_ROLL,
    selector::CORNER_TL,
    selector::CORNER_TR,
    selector::CORNER_BL,
    selector::CORNER_BR,
    selector::FLIP_SHADER,
    selector::FLIP_BACK_REVEAL,
    selector::TAG_CLOUD_OFFSET_X,
    selector::TAG_CLOUD_OFFSET_Y,
    selector::STEAM_COLUMNS,
    selector::STEAM_ROWS,
    selector::STEAM_THUMB_WIDTH,
    selector::STEAM_THUMB_HEIGHT,
    selector::WALLHAVEN_COLUMNS,
    selector::WALLHAVEN_ROWS,
    selector::WALLHAVEN_THUMB_WIDTH,
    selector::WALLHAVEN_THUMB_HEIGHT,
    "filterBar.offsetX",
    "filterBar.offsetY",
    // Transition internals.
    transition::FPS,
    transition::PREVIEW_FPS,
    transition::SAND_QUALITY,
    transition::SAND_SCOPE,
    transition::SAND_SHARP,
    transition::SAND_PRIMARY,
    // Wallpaper engine internals.
    paper::LOAD_TIMEOUT_SECONDS,
    paper::VIDEO_MULTI_PROCESS,
    paper::VIDEO_ENGINE,
    paper::WALLPAPER_LAYER,
    we_render::SCALING,
    // Performance internals.
    performance::MAX_THUMB_JOBS,
    performance::GPU_DEVICE,
    performance::IMAGE_OPTIMIZE_PRESET,
    performance::IMAGE_OPTIMIZE_RESOLUTION,
    performance::IMAGE_TRASH_DAYS,
    performance::BATTERY_VIDEO_IDLE_SECONDS,
    performance::BATTERY_WALLPAPER_PERFORMANCE,
    library::POLLING_FALLBACK,
    library::POLLING_INTERVAL_SECONDS,
    // Timing and behaviour details.
    general::OPEN_FADE_FROM,
    general::LOCALE,
    motion::FILTER_SWAP_SPEED,
    motion::LAUNCH_SPEED,
    playback::RESUME_DELAY,
    playback::FULLSCREEN_SCOPE,
    system::PICK_ONLY_MODE,
    system::POST_PROCESS_ON_RESTORE,
    system::EXTERNAL_MATUGEN_COMMAND,
    // Weather matching details.
    weather::SEMANTIC_DWELL_SECONDS,
    weather::SEMANTIC_MAX_RESULTS,
    weather::SEMANTIC_SELECTION,
    semantic::INDEX_PROFILE,
    semantic::MANIFEST,
    // Expert colour options.
    matugen::COLOR_INDEX,
    matugen::CONTRAST,
    matugen::DEFAULT_CONFIG,
    theme::WALLUST_PALETTE,
    theme::AUTHORITY,
    niri::BACKDROP_BLUR_STATIC_RADIUS,
    niri::BACKDROP_BLUR_VIDEO_RADIUS,
    niri::BACKDROP_BLUR_WE_RADIUS,
    // Download limits.
    sources::YOUTUBE_MAX_HEIGHT,
    sources::YOUTUBE_MAX_MINUTES,
    sources::BING_MARKET,
];

/// Prefixes of advanced paths (per-display transition scopes, awww).
const ADVANCED_PREFIXES: &[&str] = &["transition.shaderScopes.", "paper.awww"];

pub fn is_advanced_path(path: &str) -> bool {
    ADVANCED.contains(&path) || ADVANCED_PREFIXES.iter().any(|prefix| path.starts_with(prefix))
}

pub fn row_is_advanced(row: &Row) -> bool {
    super::palette::control_path(&row.control).is_some_and(|path| is_advanced_path(&path))
        || matches!(&row.control, Control::Code { .. })
}

#[cfg(test)]
pub(super) fn advanced_paths() -> &'static [&'static str] {
    ADVANCED
}
