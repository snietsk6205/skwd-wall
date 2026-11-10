use super::icons::{
    ICON_APPS, ICON_AUDIO, ICON_BAR, ICON_BASICS, ICON_BATTERY, ICON_BELL, ICON_BUG, ICON_CARDS,
    ICON_COLOURS, ICON_CONSOLE, ICON_DESIGNER, ICON_DISPLAYS, ICON_EXCLUDED, ICON_FILL,
    ICON_FILTER, ICON_FOLDER, ICON_GAUGE, ICON_HOME, ICON_KEYS, ICON_LANGUAGE, ICON_LAUNCH,
    ICON_LAYOUT, ICON_LEAF, ICON_LIBRARY, ICON_LOOK, ICON_MOTION, ICON_PAUSE, ICON_PLAYBACK,
    ICON_POSITION, ICON_PROFILES, ICON_PUZZLE, ICON_QUALITY, ICON_SAND, ICON_SCHEDULE, ICON_SEARCH,
    ICON_SHUFFLE, ICON_SKETCH, ICON_SOURCES, ICON_SPEED, ICON_STEAM, ICON_SYSTEM, ICON_TAGS,
    ICON_VIDEO, ICON_WALLPAPERS, ICON_WEATHER, ICON_WIND, ICON_WINDOW,
};

/// A section title: a locale key (upstream sections) or English text (custom
/// build sections).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Title {
    Key(&'static str),
    Text(&'static str),
}

impl Title {
    pub fn text(self) -> &'static str {
        match self {
            Self::Key(key) => crate::i18n::tr(key),
            Self::Text(text) => text,
        }
    }
}

/// Which old Settings sections a page shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ref {
    /// The section(s) of this tab with this title.
    Section(&'static str, Title),
    /// Every section of this tab that no other page names.
    Rest(&'static str),
}

impl Ref {
    pub const fn tab(self) -> &'static str {
        match self {
            Self::Section(tab, _) | Self::Rest(tab) => tab,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Group {
    pub key: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
    pub sentence: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page {
    /// "group/page", used by the Home cards, `skwd-helm ui page` and the
    /// last-page setting.
    pub key: &'static str,
    pub group: usize,
    pub icon: &'static str,
    pub label: &'static str,
    pub sentence: &'static str,
    pub refs: &'static [Ref],
}

/// Group 0 is Home (no menu); the other five open a menu of pages.
pub const GROUPS: [Group; 6] = [
    Group {
        key: "home",
        label: "Home",
        icon: ICON_HOME,
        sentence: "Your pinned settings and the latest changes.",
    },
    Group {
        key: "look",
        label: "Look",
        icon: ICON_LOOK,
        sentence: "Picker layouts, cards, the bar, colours and theme.",
    },
    Group {
        key: "wallpapers",
        label: "Wallpapers",
        icon: ICON_WALLPAPERS,
        sentence: "Folders, online sources, displays and how pictures fill them.",
    },
    Group {
        key: "motion",
        label: "Motion",
        icon: ICON_MOTION,
        sentence: "Wallpaper transitions, the sketch and picker animations.",
    },
    Group {
        key: "playback",
        label: "Playback & Power",
        icon: ICON_PLAYBACK,
        sentence: "Videos, sound, speed and battery.",
    },
    Group {
        key: "system",
        label: "Automation & System",
        icon: ICON_SYSTEM,
        sentence: "Rotation, schedule, weather, keys, language and other apps.",
    },
];

const fn key(tab: &'static str, key: &'static str) -> Ref {
    Ref::Section(tab, Title::Key(key))
}

const fn text(tab: &'static str, text: &'static str) -> Ref {
    Ref::Section(tab, Title::Text(text))
}

const fn entry(
    key: &'static str,
    group: usize,
    icon: &'static str,
    label: &'static str,
    sentence: &'static str,
    refs: &'static [Ref],
) -> Page {
    Page { key, group, icon, label, sentence, refs }
}

pub const PAGES: [Page; 44] = [
    // Look
    entry(
        "look/layout",
        1,
        ICON_LAYOUT,
        "Layout studio",
        "Pick a layout and shape it while you watch the picker.",
        &[key("picker", "settings-section-layout-presets")],
    ),
    entry(
        "look/basics",
        1,
        ICON_BASICS,
        "Picker basics",
        "How the picker opens, closes and where it starts.",
        &[key("picker", "settings-section-general")],
    ),
    entry(
        "look/cards",
        1,
        ICON_CARDS,
        "Cards & previews",
        "Card previews, video previews and the selection ring.",
        &[key("picker", "settings-section-cards-previews"), text("picker", "Selection")],
    ),
    entry(
        "look/positions",
        1,
        ICON_POSITION,
        "Positions",
        "Where each layout sits on the screen.",
        &[
            key("position", "settings-position-slices-card"),
            key("position", "settings-position-hex-card"),
            key("position", "settings-position-wall-card"),
            key("position", "settings-position-sandy-card"),
            key("position", "settings-position-hand-card"),
        ],
    ),
    entry(
        "look/bar",
        1,
        ICON_BAR,
        "Bar",
        "Size, place, style, buttons and order of the bar.",
        &[
            text("picker", "Bar"),
            key("filter", "settings-section-appearance"),
            key("filter", "settings-section-buttons"),
            text("picker", "Bar order"),
            key("position", "settings-filter-position-card"),
        ],
    ),
    entry(
        "look/tags",
        1,
        ICON_TAGS,
        "Tag panel",
        "The panel with tags and search under the cards.",
        &[
            key("picker", "settings-section-tag-panel"),
            key("position", "settings-selector-tag-cloud-position-card"),
        ],
    ),
    entry(
        "look/window",
        1,
        ICON_WINDOW,
        "Settings window",
        "Size and place of this window, its menus and Home.",
        &[text("picker", "Settings window")],
    ),
    entry(
        "look/appearance",
        1,
        ICON_LOOK,
        "Settings appearance",
        "Cards, the grid pattern, the top bar, menus and animations of this window.",
        &[text("picker", "Settings appearance")],
    ),
    entry(
        "look/colours",
        1,
        ICON_COLOURS,
        "Colours & theme",
        "Take colours from the wallpaper for your desktop.",
        &[Ref::Rest("theme")],
    ),
    entry(
        "look/apps",
        1,
        ICON_APPS,
        "App themes",
        "Send the colours to apps like terminals and GTK.",
        &[key("theme", "settings-app-themes-title")],
    ),
    entry(
        "look/designer",
        1,
        ICON_DESIGNER,
        "Theme designer",
        "Design your own colour theme by hand.",
        &[key("theme", "settings-theme-designer-label")],
    ),
    // Wallpapers
    entry(
        "wallpapers/folders",
        2,
        ICON_FOLDER,
        "Folders",
        "The folders your wallpapers come from.",
        &[key("library", "settings-section-folders")],
    ),
    entry(
        "wallpapers/excluded",
        2,
        ICON_EXCLUDED,
        "Excluded folders",
        "Folders the picker should never show.",
        &[key("library", "settings-paths-excluded-card")],
    ),
    entry(
        "wallpapers/profiles",
        2,
        ICON_PROFILES,
        "Profiles",
        "Named sets of folders you can switch between.",
        &[key("library", "settings-profiles-card")],
    ),
    entry(
        "wallpapers/displays",
        2,
        ICON_DISPLAYS,
        "Displays",
        "What each screen shows, how it fills and locks.",
        &[Ref::Rest("displays")],
    ),
    entry(
        "wallpapers/fill",
        2,
        ICON_FILL,
        "Fill & engine",
        "How pictures fill the screen and what draws them.",
        &[key("playback", "settings-section-display")],
    ),
    entry(
        "wallpapers/sources",
        2,
        ICON_SOURCES,
        "Online sources",
        "Wallhaven, Unsplash, Pexels, Bing, Workshop and YouTube.",
        &[Ref::Rest("sources")],
    ),
    entry(
        "wallpapers/filters",
        2,
        ICON_FILTER,
        "Filters & defaults",
        "Which wallpapers show and what the picker starts with.",
        &[
            key("filter", "settings-section-defaults"),
            key("filter", "settings-section-visibility"),
            key("filter", "settings-section-resolution-presets"),
        ],
    ),
    entry(
        "wallpapers/search",
        2,
        ICON_SEARCH,
        "Search & tags",
        "Find wallpapers by tags or by describing them.",
        &[
            key("filter", "settings-section-search-tagging"),
            key("filter", "settings-section-semantic-models"),
        ],
    ),
    entry(
        "wallpapers/library",
        2,
        ICON_LIBRARY,
        "Library care",
        "Watching folders, thumbnails, cache and recovery.",
        &[
            key("library", "settings-section-library-watching"),
            key("library", "settings-section-images-recovery"),
            key("library", "settings-section-generated-variants"),
            key("library", "settings-section-thumbnails-cache"),
        ],
    ),
    // Motion
    entry(
        "motion/transitions",
        3,
        ICON_MOTION,
        "Transitions",
        "How one wallpaper changes into the next.",
        &[key("motion", "settings-section-wallpaper-transitions")],
    ),
    entry(
        "motion/sketch",
        3,
        ICON_SKETCH,
        "Sketch",
        "Draw the new wallpaper as a sketch before it appears.",
        &[text("motion", "Sketch before transition")],
    ),
    entry(
        "motion/speeds",
        3,
        ICON_SPEED,
        "Animation speeds",
        "How fast the picker's animations run.",
        &[key("motion", "settings-section-shared-speeds")],
    ),
    entry(
        "motion/opening",
        3,
        ICON_LAUNCH,
        "Opening animation",
        "How the cards appear when the picker opens.",
        &[key("motion", "settings-section-launch")],
    ),
    entry(
        "motion/picker",
        3,
        ICON_CARDS,
        "Picker motion",
        "Card flips and how filters move the cards.",
        &[key("motion", "settings-section-picker-motion")],
    ),
    entry(
        "motion/sand",
        3,
        ICON_SAND,
        "Sand & Sandy",
        "The sand transition and the Sandy layout's grains.",
        &[key("motion", "settings-section-sandy-motion")],
    ),
    entry(
        "motion/quality",
        3,
        ICON_QUALITY,
        "Transition quality",
        "Frame rate and detail of the transitions.",
        &[key("motion", "settings-section-transition-performance")],
    ),
    entry(
        "motion/awww",
        3,
        ICON_WIND,
        "awww transitions",
        "Transitions when awww draws your wallpapers.",
        &[key("motion", "settings-section-awww-transition")],
    ),
    // Playback & Power
    entry(
        "playback/video",
        4,
        ICON_VIDEO,
        "Video",
        "How video wallpapers are decoded and played.",
        &[key("playback", "settings-section-video")],
    ),
    entry(
        "playback/audio",
        4,
        ICON_AUDIO,
        "Sound",
        "Volume and when wallpapers go quiet.",
        &[key("playback", "settings-section-audio")],
    ),
    entry(
        "playback/pause",
        4,
        ICON_PAUSE,
        "Pause rules",
        "When moving wallpapers stop to save power.",
        &[key("playback", "settings-playback-pause-title")],
    ),
    entry(
        "playback/workshop",
        4,
        ICON_STEAM,
        "Wallpaper Engine",
        "How Wallpaper Engine scenes are drawn.",
        &[key("playback", "settings-section-workshop-rendering")],
    ),
    entry(
        "playback/power",
        4,
        ICON_BATTERY,
        "Power & battery",
        "Low-power mode, battery saver and the overlay.",
        &[key("performance", "settings-section-power")],
    ),
    entry(
        "playback/speed",
        4,
        ICON_GAUGE,
        "Picker speed",
        "Frame rate, graphics card and picker drawing.",
        &[key("performance", "settings-section-picker-rendering")],
    ),
    entry(
        "playback/efficiency",
        4,
        ICON_LEAF,
        "Wallpaper efficiency",
        "Keep moving wallpapers light on your computer.",
        &[key("performance", "settings-section-wallpaper-efficiency")],
    ),
    entry(
        "playback/report",
        4,
        ICON_BUG,
        "Bug report",
        "Collect what is needed to report a problem.",
        &[key("performance", "settings-performance-bug-report-label")],
    ),
    // Automation & System
    entry(
        "system/rotation",
        5,
        ICON_SHUFFLE,
        "Random rotation",
        "Change the wallpaper by itself every so often.",
        &[key("automation", "settings-section-random-rotation")],
    ),
    entry(
        "system/schedule",
        5,
        ICON_SCHEDULE,
        "Schedule & playlists",
        "Wallpapers by time of day, and your playlists.",
        &[key("automation", "settings-section-schedule")],
    ),
    entry(
        "system/weather",
        5,
        ICON_WEATHER,
        "Location & weather",
        "Match wallpapers to the weather outside.",
        &[key("automation", "settings-section-location-weather")],
    ),
    entry(
        "system/startup",
        5,
        ICON_BELL,
        "Startup & notifications",
        "What happens when you log in, and messages.",
        &[key("automation", "settings-section-startup-notifications")],
    ),
    entry(
        "system/after",
        5,
        ICON_CONSOLE,
        "After a change",
        "Commands that run after a new wallpaper is set.",
        &[
            key("automation", "settings-section-post-apply-behaviour"),
            key("automation", "settings-section-post-apply-commands"),
        ],
    ),
    entry(
        "system/keys",
        5,
        ICON_KEYS,
        "Keys & controls",
        "Change any keyboard shortcut or mouse button.",
        &[key("picker", "settings-section-controls")],
    ),
    entry(
        "system/language",
        5,
        ICON_LANGUAGE,
        "Language",
        "The language of the picker and Settings.",
        &[Ref::Rest("language")],
    ),
    entry(
        "system/integrations",
        5,
        ICON_PUZZLE,
        "Desktop & lock screen",
        "Shells, lock screens and other apps.",
        &[Ref::Rest("integrations")],
    ),
];

pub fn page_index(key: &str) -> Option<usize> {
    PAGES.iter().position(|page| page.key == key)
}

pub fn group_pages(group: usize) -> impl Iterator<Item = (usize, &'static Page)> {
    PAGES.iter().enumerate().filter(move |(_, page)| page.group == group)
}

/// The `&'static` name of an old tab ("picker" when unknown).
pub fn static_tab(tab: &str) -> &'static str {
    super::super::tables::TABS.iter().copied().find(|known| *known == tab).unwrap_or("picker")
}

/// A page by key, "group/page", or a group key alone (its first page).
pub fn find_page(key: &str) -> Option<usize> {
    let key = key.trim().trim_matches('/');
    page_index(key)
        .or_else(|| PAGES.iter().position(|page| page.key.rsplit('/').next() == Some(key)))
        .or_else(|| {
            let group = GROUPS.iter().position(|group| group.key == key)?;
            PAGES.iter().position(|page| page.group == group)
        })
}
