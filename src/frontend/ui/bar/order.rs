//! Custom build: the order of the bar's button groups (`filterBar.order`).

/// Every button group, in upstream's order.
pub const BAR_GROUPS: [&str; 18] = [
    "types",
    "orient",
    "resolution",
    "folder",
    "hidden",
    "sorts",
    "favourites",
    "random",
    "colors",
    "theme",
    "tagcloud",
    "download",
    "audio",
    "playlists",
    "effects",
    "layout",
    "settings",
    "stash",
];

/// Readable name of a group for the Settings window.
pub fn bar_group_label(group: &str) -> &'static str {
    match group {
        "types" => "Type filters (Images, Videos, ...)",
        "orient" => "Shape filter",
        "resolution" => "Resolution filter",
        "folder" => "Folder menu",
        "hidden" => "Hidden wallpapers",
        "sorts" => "Sort buttons",
        "favourites" => "Favourites",
        "random" => "Random rotation",
        "colors" => "Colour filter",
        "theme" => "Theme panel",
        "tagcloud" => "Tag cloud",
        "download" => "Download (Wallhaven, Steam)",
        "audio" => "Audio",
        "playlists" => "Playlists",
        "effects" => "Effects studio",
        "layout" => "Switch layout",
        "settings" => "Settings",
        "stash" => "Hide picker",
        _ => "Unknown",
    }
}

/// Parses a comma-separated list. Unknown and repeated names are dropped and
/// groups that are missing are added in upstream's order, so every group
/// always appears exactly once.
pub fn bar_order(text: &str) -> Vec<&'static str> {
    let mut order: Vec<&'static str> = Vec::with_capacity(BAR_GROUPS.len());
    for name in text.split(',').map(str::trim) {
        if let Some(group) = BAR_GROUPS.iter().find(|group| **group == name)
            && !order.contains(group)
        {
            order.push(group);
        }
    }
    for group in BAR_GROUPS {
        if !order.contains(&group) {
            order.push(group);
        }
    }
    order
}

/// The order after moving `group` one place earlier or later.
pub fn bar_order_moved(text: &str, group: &str, earlier: bool) -> String {
    let mut order = bar_order(text);
    if let Some(at) = order.iter().position(|name| *name == group) {
        let to = if earlier { at.checked_sub(1) } else { Some(at + 1) };
        if let Some(to) = to.filter(|to| *to < order.len()) {
            order.swap(at, to);
        }
    }
    order.join(",")
}

#[cfg(test)]
#[path = "order_tests.rs"]
mod tests;
