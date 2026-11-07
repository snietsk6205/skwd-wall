//! Custom build: Settings cards of the Coverflow and Showcase layouts
//! (Settings → Picker → Layout, shown when that layout is selected).

use crate::contracts::settings::keys;
use crate::i18n::tr;

use super::{Builder, Control};

const COVERFLOW_STYLES: [(&str, &str); 3] =
    [("coverflow", "Coverflow"), ("carousel", "Carousel ring"), ("stack", "Stack")];

const STRIP_POSITIONS: [(&str, &str); 2] = [("bottom", "Bottom"), ("top", "Top")];

pub(super) fn chips_or_first(path: &str, options: &[(&str, &str)], current: &str) -> Control {
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

pub(super) fn coverflow_cards(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    builder.card(
        tr("settings-selector-mode-coverflow"),
        "Cards in 3D. Left/Right, the wheel or a click moves; click the front card or press Enter to apply.",
    );
    let style = cfg.text(keys::selector::COVERFLOW_STYLE);
    builder.row(
        "Style",
        "Coverflow: the front card faces you, the others turn away. Carousel ring: the cards stand on a turning ring. Stack: the cards recede behind the front one, like a timeline.",
        chips_or_first(keys::selector::COVERFLOW_STYLE, &COVERFLOW_STYLES, &style),
    );
    builder.num(
        "Card size",
        "Card height in percent of the screen height, 20 to 75.",
        keys::selector::COVERFLOW_SIZE,
        "%",
    );
    builder.num(
        "Angle",
        "How far the side cards turn away, 0 to 85 degrees.",
        keys::selector::COVERFLOW_ANGLE,
        "°",
    );
    builder.num(
        "Spacing",
        "Distance between side cards in percent of the card width, 8 to 90.",
        keys::selector::COVERFLOW_SPACING,
        "%",
    );
    builder.num(
        "Cards on each side",
        "How many cards are visible on each side of the front card, 1 to 12.",
        keys::selector::COVERFLOW_COUNT,
        "",
    );
    builder.num(
        "Corner radius",
        "Round the card corners, 0 to 40 px.",
        keys::selector::COVERFLOW_CORNERS,
        "px",
    );
    builder.num(
        "Step time",
        "Time to move one card, 120 to 1500 ms. Lower is snappier.",
        keys::selector::COVERFLOW_SPEED,
        "ms",
    );
    builder.toggle_default_true(
        "Reflection",
        "A soft mirror image under the cards (Coverflow and Carousel ring).",
        keys::selector::COVERFLOW_REFLECTION,
    );
    builder.toggle_default_true(
        "Blurred background",
        "Show the selected wallpaper blurred and darkened behind the cards.",
        keys::selector::COVERFLOW_BACKDROP,
    );
}

pub(super) fn showcase_cards(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    builder.card(
        tr("settings-selector-mode-showcase"),
        "One big preview of the selected wallpaper and a filmstrip of the others. Left/Right or the wheel moves; click the preview or press Enter to apply.",
    );
    builder.num(
        "Preview size",
        "Width of the big preview in percent of the screen width, 35 to 85.",
        keys::selector::SHOWCASE_HERO,
        "%",
    );
    builder.num(
        "Filmstrip height",
        "Height of the small cards, 48 to 180 px.",
        keys::selector::SHOWCASE_STRIP,
        "px",
    );
    let position = cfg.text(keys::selector::SHOWCASE_STRIP_POSITION);
    builder.row(
        "Filmstrip position",
        "Show the filmstrip under or above the preview.",
        chips_or_first(keys::selector::SHOWCASE_STRIP_POSITION, &STRIP_POSITIONS, &position),
    );
    builder.num(
        "Corner radius",
        "Round the corners of the preview, 0 to 40 px.",
        keys::selector::SHOWCASE_CORNERS,
        "px",
    );
    builder.num(
        "Step time",
        "Time of the cross-fade between pictures, 120 to 1500 ms.",
        keys::selector::SHOWCASE_SPEED,
        "ms",
    );
    builder.toggle_default_true(
        "Blurred background",
        "Show the selected wallpaper blurred and darkened behind everything.",
        keys::selector::SHOWCASE_BACKDROP,
    );
}
