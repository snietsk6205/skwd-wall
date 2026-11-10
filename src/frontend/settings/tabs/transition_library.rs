//! Custom build (M3): the transition library (search, families, favourites,
//! a live preview per transition), the playlist ("play in turns") and the
//! "combine two" cards of Settings → Motion → Wallpaper transitions.
//!
//! Lists are comma lists in the config (like the bar order); the daemon reads
//! the same keys (`skwd-deck` `value/transition/deck.rs`).

use crate::contracts::settings::{SettingsSource, keys, transitions};

use super::{ActionId, Builder, Control};

pub(crate) const LIBRARY_CARD: &str = "Transition library";
pub(crate) const PLAYLIST_CARD: &str = "Play in turns";
/// Heading of upstream's own transition rows on the page.
pub(crate) const BASICS_CARD: &str = "How transitions play";
pub(crate) const COMBO_CARD: &str = "Combine two";

/// Order of a transition card's actions.
pub(crate) const TILE_USE: usize = 1;
pub(crate) const TILE_FAVOURITE: usize = 3;

/// Library filters besides the families.
const FILTERS: [(&str, &str); 3] =
    [("all", "All"), ("favourites", "Favourites"), ("playlist", "In playlist")];

/// Splits a comma list ("burn, ripple,,iris") into its entries.
pub(crate) fn transition_list(text: &str) -> Vec<String> {
    text.split(',').map(str::trim).filter(|entry| !entry.is_empty()).map(String::from).collect()
}

/// `list` with `entry` added at the end, or taken out if it is there.
pub(crate) fn list_toggled(list: &[String], entry: &str) -> Vec<String> {
    if list.iter().any(|item| item == entry) {
        list.iter().filter(|item| *item != entry).cloned().collect()
    } else {
        let mut out = list.to_vec();
        out.push(entry.to_string());
        out
    }
}

/// `list` with entry `index` moved one place earlier or later.
pub(crate) fn list_moved(list: &[String], index: usize, earlier: bool) -> Vec<String> {
    let mut out = list.to_vec();
    let target = if earlier { index.checked_sub(1) } else { Some(index + 1) };
    if let Some(target) = target.filter(|target| *target < out.len() && index < out.len()) {
        out.swap(index, target);
    }
    out
}

/// Entries that can be played or listed: real transitions plus the
/// renderer's "random-mix" (not "random", "playlist" or "combo").
fn library_entries() -> impl Iterator<Item = &'static transitions::TransitionSpec> {
    transitions::TRANSITIONS
        .iter()
        .filter(|spec| !matches!(spec.key, "random" | "playlist" | "combo"))
}

/// Whether a transition shows in the library for this search and filter.
pub(crate) fn library_matches(
    spec: &transitions::TransitionSpec,
    search: &str,
    filter: &str,
    favourites: &[String],
    playlist: &[String],
) -> bool {
    let shown = match filter {
        "favourites" => favourites.iter().any(|entry| entry == spec.key),
        "playlist" => playlist.iter().any(|entry| entry == spec.key),
        "all" | "" => true,
        family => spec.family.key() == family,
    };
    if !shown {
        return false;
    }
    let search = search.trim().to_lowercase();
    if search.is_empty() {
        return true;
    }
    let haystack = format!(
        "{} {} {} {}",
        spec.key,
        transitions::pretty(spec.key),
        spec.family.label(),
        spec.desc
    )
    .to_lowercase();
    search.split_whitespace().all(|word| haystack.contains(word))
}

fn filter_options() -> Vec<(String, String)> {
    let mut options: Vec<(String, String)> =
        FILTERS.iter().map(|(key, label)| ((*key).to_string(), (*label).to_string())).collect();
    for family in transitions::FAMILIES {
        if matches!(family.key(), "random" | "playlist") {
            continue;
        }
        options.push((family.key().to_string(), family.label().to_string()));
    }
    options
}

fn chips(path: &str, options: Vec<(String, String)>, current: String) -> Control {
    let current = if options.iter().any(|(key, _)| *key == current) {
        current
    } else {
        options.first().map(|(key, _)| key.clone()).unwrap_or_default()
    };
    Control::Chips {
        path: path.to_string(),
        options,
        current,
        disabled: Vec::new(),
        background: None,
    }
}

fn pairs(options: &[(&str, &str)]) -> Vec<(String, String)> {
    options.iter().map(|(key, label)| ((*key).to_string(), (*label).to_string())).collect()
}

/// Search, filters and one card per transition (R1b): its picture inside,
/// tags, and the actions Preview big, Use, playlist and favourite.
pub(super) fn tab_library(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    builder.card(
        LIBRARY_CARD,
        "Every transition as a card. Point at one (or move to it with the arrow keys) and it plays inside the card; Preview big shows it large until you press Esc.",
    );
    builder.text_field(
        "Search",
        "Type a name or a word from the description, for example \"light\", \"tiles\" or \"water\".",
        keys::transition::LIBRARY_SEARCH,
        "burn, glow, tiles…",
    );
    let filter = cfg.text(keys::transition::LIBRARY_FILTER);
    let filter = if filter.is_empty() { String::from("all") } else { filter };
    builder.row(
        "Show",
        "All transitions, your favourites, the ones in your playlist, or one family.",
        chips(keys::transition::LIBRARY_FILTER, filter_options(), filter.clone()),
    );
    let search = cfg.text(keys::transition::LIBRARY_SEARCH);
    let favourites = transition_list(&cfg.text(keys::transition::FAVOURITES));
    let playlist = transition_list(&cfg.text(keys::transition::PLAYLIST));
    let current = cfg.text(keys::transition::SHADER);
    let mut shown = 0usize;
    for spec in library_entries() {
        if !library_matches(spec, &search, &filter, &favourites, &playlist) {
            continue;
        }
        shown += 1;
        let starred = favourites.iter().any(|entry| entry == spec.key);
        let listed = playlist.iter().any(|entry| entry == spec.key);
        let playing = current == spec.key;
        let mut tags = vec![(spec.family.label().to_string(), false)];
        if playing {
            tags.push((String::from("Playing now"), true));
        }
        if listed {
            tags.push((String::from("In playlist"), false));
        }
        if starred && tags.len() < 3 {
            tags.push((String::from("Favourite"), false));
        }
        let title = transitions::pretty(spec.key);
        let star = if starred { "Remove favourite" } else { "Favourite" };
        let list = if listed { "In playlist ✓" } else { "+ Playlist" };
        builder.tile(
            &title,
            spec.desc,
            super::Tile {
                picture: spec.key.to_string(),
                tags,
                plus: Some((TILE_FAVOURITE, starred)),
                primary: TILE_USE,
                active: playing,
            },
            &[
                (ActionId::PreviewTransition(spec.key.to_string()), "Preview big"),
                (ActionId::UseTransition(spec.key.to_string()), "Use"),
                (ActionId::PlaylistToggle(spec.key.to_string()), list),
                (ActionId::FavouriteTransition(spec.key), star),
            ],
        );
    }
    if shown == 0 {
        let hint = match filter.as_str() {
            "favourites" => "No favourites yet: press Favourite next to a transition.",
            "playlist" => "Your playlist is empty: press + Playlist next to a transition.",
            _ => "Nothing matches this search.",
        };
        builder.info("No transitions", hint);
    }
}

fn order_desc(order: &str) -> &'static str {
    match order {
        "shuffle" => "Every transition once per round, in a new random order each round.",
        "no-repeat" => {
            "A random one each time, but never one of the last few, so none plays twice in a row."
        }
        _ => "One after the other, from the top of the list, then again from the start.",
    }
}

/// The playlist: order, entries (preview, move, remove) and quick actions.
pub(super) fn tab_playlist(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    builder.card(
        PLAYLIST_CARD,
        "Play several transitions in turns, one per wallpaper change. Add them with + Playlist in the library.",
    );
    let order = cfg.text(keys::transition::PLAYLIST_ORDER);
    let order = if order.is_empty() { String::from("in-order") } else { order };
    builder.row(
        "Order",
        order_desc(&order),
        chips(keys::transition::PLAYLIST_ORDER, pairs(&transitions::PLAYLIST_ORDERS), order),
    );
    let playlist = transition_list(&cfg.text(keys::transition::PLAYLIST));
    let active = cfg.text(keys::transition::SHADER) == "playlist";
    if !active {
        builder.action(
            "Use the playlist",
            "Right now one transition (or Random) plays. Switch to playing this list in turns.",
            ActionId::UseTransition(String::from("playlist")),
            "Play in turns",
        );
    }
    if playlist.is_empty() {
        builder.info(
            "The list is empty",
            "While it is empty, Random plays. Add transitions with + Playlist in the library, or add your favourites.",
        );
    }
    for (index, entry) in playlist.iter().enumerate() {
        let spec = transitions::spec(entry);
        let desc = spec.map_or("A combination of two transitions.", |spec| spec.desc);
        builder.action_chips(
            &format!("{}. {}", index + 1, transitions::label_of(entry)),
            desc,
            &[
                (ActionId::PlaylistPreview(index), "▶ Preview"),
                (ActionId::PlaylistMove(index, true), "◀ Earlier"),
                (ActionId::PlaylistMove(index, false), "Later ▶"),
                (ActionId::PlaylistRemove(index), "× Remove"),
            ],
        );
    }
    builder.action_chips(
        "Quick changes",
        "Add every favourite that is not in the list yet, or empty the list.",
        &[
            (ActionId::PlaylistAddFavourites, "+ All favourites"),
            (ActionId::PlaylistClear, "Clear"),
        ],
    );
}

fn mode_desc(mode: &str) -> &'static str {
    match mode {
        "blend" => "Both run at the same time; the picture turns from the first into the second.",
        "split" => "The first plays on one part of the screen, the second on the other.",
        "mask" => {
            "The first decides where the change happens (its shape), the second how it looks inside."
        }
        "sweep" => "A soft line moves across: the first plays ahead of it, the second behind it.",
        "light" => "Both at once; wherever one is brighter, it shows.",
        "dark" => "Both at once; wherever one is darker, it shows.",
        _ => {
            "The first plays to its half-way point, that moment is frozen, and the second finishes from it."
        }
    }
}

/// Families a combined transition can come from (not playlist or combo).
fn combo_families() -> Vec<(String, String)> {
    transitions::FAMILIES
        .iter()
        .filter(|family| !matches!(family.key(), "playlist" | "combo"))
        .map(|family| (family.key().to_string(), family.label().to_string()))
        .collect()
}

/// The family of a combined transition ("random" for a random one).
pub(crate) fn combo_part_family(key: &str) -> &'static str {
    if key == "random" { "random" } else { transitions::family_of(key).key() }
}

/// The transitions of one family a combination can use.
fn combo_parts(family: &str) -> Vec<(String, String)> {
    if family == "random" {
        return vec![(String::from("random"), String::from("Random"))];
    }
    library_entries()
        .filter(|spec| spec.family.key() == family && spec.key != "random-mix")
        .map(|spec| (spec.key.to_string(), transitions::pretty(spec.key)))
        .collect()
}

/// Two rows for one part: its family, then a transition of that family.
fn combo_part_rows(
    builder: &mut Builder<'_>,
    (title, desc): (&str, &str),
    path: &str,
    family_path: &str,
    fallback: &str,
) {
    let value = builder.cfg.text(path);
    let value = if transitions::is_shader(&value) || value == "random" {
        value
    } else {
        fallback.to_string()
    };
    let family = combo_part_family(&value);
    builder.row(
        &format!("{title}: family"),
        desc,
        chips(family_path, combo_families(), family.to_string()),
    );
    builder.row(title, "", chips(path, combo_parts(family), value));
}

fn known(
    cfg: &dyn SettingsSource,
    path: &str,
    fallback: &str,
    options: &[(String, String)],
) -> String {
    let value = cfg.text(path);
    if options.iter().any(|(key, _)| *key == value) { value } else { fallback.to_string() }
}

/// The renderer name of the configured combination (also used by the preview).
pub(crate) fn transition_combo_name(cfg: &dyn SettingsSource) -> String {
    let text = |path: &str, fallback: &str| {
        let value = cfg.text(path);
        if value.is_empty() { fallback.to_string() } else { value }
    };
    transitions::combo_name(
        &text(keys::transition::COMBO_MODE, "then"),
        &text(keys::transition::COMBO_SPLIT, "lr"),
        &text(keys::transition::COMBO_FIRST, "burn"),
        &text(keys::transition::COMBO_SECOND, "ripple"),
    )
}

/// The "combine two" card: first, second, how, split shape and actions.
pub(super) fn tab_combo(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    builder.card(
        COMBO_CARD,
        "Make your own transition out of two: one after the other, blended, split screen, one as a mask for the other, and more.",
    );
    combo_part_rows(
        builder,
        ("First transition", "Starts the change (in \"mask\": gives the shape)."),
        keys::transition::COMBO_FIRST,
        keys::transition::COMBO_FIRST_FAMILY,
        "burn",
    );
    combo_part_rows(
        builder,
        ("Second transition", "Finishes the change (in \"mask\": gives the look)."),
        keys::transition::COMBO_SECOND,
        keys::transition::COMBO_SECOND_FAMILY,
        "ripple",
    );
    let modes = pairs(&transitions::COMBO_MODES);
    let mode = known(cfg, keys::transition::COMBO_MODE, "then", &modes);
    builder.row("How", mode_desc(&mode), chips(keys::transition::COMBO_MODE, modes, mode.clone()));
    if mode == "split" {
        let splits = pairs(&transitions::COMBO_SPLITS);
        let split = known(cfg, keys::transition::COMBO_SPLIT, "lr", &splits);
        builder.row(
            "Split shape",
            "Where the first transition plays: left half, top half, one corner, inside the circle (around the start point) or every other square.",
            chips(keys::transition::COMBO_SPLIT, splits, split),
        );
    }
    let name = transition_combo_name(cfg);
    let playing = cfg.text(keys::transition::SHADER) == "combo";
    let listed = transition_list(&cfg.text(keys::transition::PLAYLIST)).contains(&name);
    builder.action_chips(
        &transitions::label_of(&name),
        if playing {
            "This combination plays now. Add it to the playlist to keep it next to others."
        } else {
            "Preview it, make it the transition that plays, or keep it in the playlist."
        },
        &[
            (ActionId::PreviewTransition(String::from("combo")), "▶ Preview"),
            (ActionId::UseTransition(String::from("combo")), "Use"),
            (
                ActionId::PlaylistToggle(name.clone()),
                if listed { "✓ In playlist" } else { "+ Playlist" },
            ),
            (ActionId::ComboSwap, "Swap first and second"),
        ],
    );
}

mod tests;
