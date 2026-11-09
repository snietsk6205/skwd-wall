//! Custom build: how the selected card is marked and how big, where and how
//! dense the Settings window is (Settings → Picker).

use crate::contracts::settings::keys;

use super::{ActionId, Builder, Control};

const POSITIONS: [(&str, &str); 5] = [
    ("centre", "Centre"),
    ("left", "Left"),
    ("right", "Right"),
    ("top", "Top"),
    ("bottom", "Bottom"),
];

const OPEN_ON: [(&str, &str); 2] = [("home", "Home"), ("last", "Where I left off")];
const MENU_STYLES: [(&str, &str); 2] = [("list", "List"), ("grid", "Grid")];
const VIEWS: [(&str, &str); 2] = [("simple", "Simple"), ("advanced", "Advanced")];
const BACKGROUNDS: [(&str, &str); 3] =
    [("wallpaper", "Current wallpaper"), ("custom", "A picture or video"), ("none", "None")];

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

const BAR_ANCHORS: [(&str, &str); 5] = [
    ("auto", "Automatic"),
    ("top", "Top"),
    ("bottom", "Bottom"),
    ("left", "Left"),
    ("right", "Right"),
];

fn tab_bar(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    builder.card(
        "Bar",
        "Size, place and button order of the bar above the wallpapers. Its style (Pill, Glass, ...) is under Filter & search → Appearance, and which buttons show under Filter & search → Buttons.",
    );
    builder.num(
        "Size",
        "Size of the bar and its buttons, 0.6 to 1.8 (1 = normal).",
        keys::filter_bar::SCALE,
        "x",
    );
    let anchor = cfg.text(keys::filter_bar::ANCHOR);
    builder.row(
        "Position",
        "Automatic keeps the bar next to the cards. Top and Bottom pin it to that screen edge; Left and Right make it a vertical rail on that side.",
        chips(keys::filter_bar::ANCHOR, &BAR_ANCHORS, &anchor),
    );
    builder.action_chips(
        "Quick setup",
        "Minimal keeps only the folder menu, favourites, effects, layout switch, settings and the type and sort buttons. Everything turns every button back on. Single buttons are under Filter & search → Buttons.",
        &[(ActionId::BarPreset(true), "Minimal"), (ActionId::BarPreset(false), "Everything")],
    );
    builder.card(
        "Bar order",
        "Move button groups earlier (left or up) or later (right or down). Groups that are switched off keep their place.",
    );
    let order = crate::frontend::ui::bar_order(&cfg.text(keys::filter_bar::ORDER));
    for (place, group) in order.iter().enumerate() {
        let title = format!("{}. {}", place + 1, crate::frontend::ui::bar_group_label(group));
        builder.action_chips(
            &title,
            "",
            &[
                (ActionId::MoveBarGroup(group, true), "◀ Earlier"),
                (ActionId::MoveBarGroup(group, false), "Later ▶"),
            ],
        );
    }
    builder.action_chips(
        "Reset order",
        "Put the groups back in the original order.",
        &[(ActionId::ResetBarOrder, "Reset")],
    );
}

pub(super) fn tab_window(builder: &mut Builder<'_>) {
    use keys::settings_window as window;
    tab_bar(builder);
    let cfg = builder.cfg;
    builder.card("Selection", "How the selected card is marked while you use the keyboard.");
    builder.num(
        "Selection ring",
        "Width of the ring around the selected card in Grid mode, 0 to 12 px (default 3).",
        keys::selector::GRID_FOCUS_WIDTH,
        "px",
    );
    builder.card(
        "Settings window",
        "Size, place and scale of this window and of every panel that opens over the picker: layout studio, playlists, card picker, theme designer, effects studio, schedule, tag cloud, scene properties, downloads and audio. The picker itself always fills the screen.",
    );
    builder.num(
        "Width",
        "Share of the screen width, 20 to 100 %. 0 keeps each panel's automatic size. The layout studio stops at about half the screen so you can still see the picker.",
        keys::settings_window::WIDTH_PERCENT,
        "%",
    );
    builder.num(
        "Height",
        "Share of the screen height, 20 to 100 %. 0 keeps each panel's automatic size.",
        keys::settings_window::HEIGHT_PERCENT,
        "%",
    );
    builder.num(
        "Scale",
        "Size of text and controls in this window and the panels, 0.5 to 2. 0 follows the UI scale.",
        keys::settings_window::SCALE,
        "x",
    );
    let position = cfg.text(keys::settings_window::POSITION);
    builder.row(
        "Position",
        "Where the window and the panels sit on the screen. The layout studio uses Left or Right for its side; Centre keeps the tag cloud under the cards.",
        chips(keys::settings_window::POSITION, &POSITIONS, &position),
    );
    builder.row(
        "Open Settings on",
        "Home shows your pinned cards each time. Where I left off opens the page you used last.",
        chips(window::OPEN_ON, &OPEN_ON, &cfg.text(window::OPEN_ON)),
    );
    builder.row(
        "Menu style",
        "How a group in the top bar shows its pages: a list (icon, name and sentence) or a grid of tiles.",
        chips(window::MENU_STYLE, &MENU_STYLES, &cfg.text(window::MENU_STYLE)),
    );
    builder.row(
        "View",
        "Simple hides the advanced settings; Advanced shows every setting. The switch in the top bar does the same. Search (Ctrl+K) always finds everything.",
        chips(window::VIEW, &VIEWS, &cfg.text(window::VIEW)),
    );
    builder.toggle(
        "Blurred top bar",
        "Put a soft, blurred copy of the Home picture behind the top bar.",
        window::BAR_BLUR,
    );
    builder.row(
        "Home background",
        "The living picture behind Home: your current wallpaper (a video plays), a picture or video you choose, or none.",
        chips(window::HOME_BACKGROUND, &BACKGROUNDS, &cfg.text(window::HOME_BACKGROUND)),
    );
    builder.text_field(
        "Background file",
        "With \"A picture or video\": the full path of the file, for example ~/Pictures/sea.mp4.",
        window::HOME_BACKGROUND_PATH,
        "~/Pictures/...",
    );
    builder.action(
        "Use the selected wallpaper",
        "Takes the wallpaper selected in the picker as the Home background.",
        ActionId::HomeUseSelected,
        "Use selected",
    );
    builder.toggle(
        "Pause the background",
        "Keeps a video background still. It also stops by itself in Low-power mode, with Battery saver, and whenever Settings is closed.",
        window::HOME_PAUSED,
    );
    builder.action(
        "Home cards",
        "Add, remove and reorder the cards on Home.",
        ActionId::EditHomeCards,
        "Edit cards",
    );
}
