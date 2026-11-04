//! Custom build: how the selected card is marked and how big, where and how
//! dense the Settings window is (Settings → Picker).

use crate::contracts::settings::keys;

use super::{Builder, Control};

const POSITIONS: [(&str, &str); 5] = [
    ("centre", "Centre"),
    ("left", "Left"),
    ("right", "Right"),
    ("top", "Top"),
    ("bottom", "Bottom"),
];

const COLUMNS: [(&str, &str); 2] = [("auto", "Automatic"), ("one", "One column")];

fn chips(path: &str, options: &[(&str, &str)], current: &str) -> Control {
    let current = if options.iter().any(|(key, _)| *key == current) {
        current.to_string()
    } else {
        options[0].0.to_string()
    };
    Control::Chips {
        path: path.to_string(),
        options: options
            .iter()
            .map(|(key, label)| ((*key).to_string(), (*label).to_string()))
            .collect(),
        current,
        disabled: Vec::new(),
        background: None,
    }
}

pub(super) fn tab_window(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    builder.card("Selection", "How the selected card is marked while you use the keyboard.");
    builder.num(
        "Selection ring",
        "Width of the ring around the selected card in Grid mode, 0 to 12 px (default 3).",
        keys::selector::GRID_FOCUS_WIDTH,
        "px",
    );
    builder.card("Settings window", "Size, place and layout of this window.");
    builder.num(
        "Width",
        "Share of the screen width, 20 to 100 %. 0 keeps the automatic size.",
        keys::settings_window::WIDTH_PERCENT,
        "%",
    );
    builder.num(
        "Height",
        "Share of the screen height, 20 to 100 %. 0 keeps the automatic size.",
        keys::settings_window::HEIGHT_PERCENT,
        "%",
    );
    builder.num(
        "Scale",
        "Size of text and controls in this window, 0.5 to 2. 0 follows the UI scale.",
        keys::settings_window::SCALE,
        "x",
    );
    let position = cfg.text(keys::settings_window::POSITION);
    builder.row(
        "Position",
        "Where the window sits on the screen.",
        chips(keys::settings_window::POSITION, &POSITIONS, &position),
    );
    let columns = cfg.text(keys::settings_window::COLUMNS);
    builder.row(
        "Columns",
        "Automatic shows two columns of settings when there is room; One column is easier to read in a small window.",
        chips(keys::settings_window::COLUMNS, &COLUMNS, &columns),
    );
    builder.toggle(
        "Narrow index",
        "A slimmer list of pages on the left, leaving more room for the settings.",
        keys::settings_window::COMPACT_INDEX,
    );
}
