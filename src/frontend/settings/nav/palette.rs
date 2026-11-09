//! R1: the command palette's index (every setting, page, transition, layout
//! and action) and its fuzzy matching. Pure data; the app runs the results.

use super::super::{Control, Row};
use super::icons::{ICON_ACTION, ICON_LAYOUT, ICON_MOTION};
use super::map::{GROUPS, PAGES};
use crate::contracts::settings::transitions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteKind {
    Page,
    Setting,
    Transition,
    Layout,
    Action,
}

impl PaletteKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Page => "Page",
            Self::Setting => "Setting",
            Self::Transition => "Transition",
            Self::Layout => "Layout",
            Self::Action => "Action",
        }
    }
}

/// What a result can change in place.
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteValue {
    None,
    Toggle { path: String, value: bool },
    Choice { path: String, options: Vec<(String, String)>, current: String },
}

/// What Enter does with a result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaletteAction {
    /// Open page `PAGES[index]`.
    Page(usize),
    /// Open page `page` and focus row `row` of its full row list.
    Setting { page: usize, row: usize },
    /// Make this transition the one that plays.
    Transition(&'static str),
    /// Switch the picker to this layout.
    Layout(&'static str),
    /// One of `palette_actions()`.
    Run(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PaletteEntry {
    pub id: String,
    pub kind: PaletteKind,
    pub icon: &'static str,
    pub title: String,
    /// Where it lives ("Motion › Transitions › Combine two").
    pub crumb: String,
    pub desc: String,
    pub value: PaletteValue,
    pub action: PaletteAction,
    haystack: String,
}

/// Actions the palette offers: (id, title, sentence).
pub fn palette_actions() -> &'static [(&'static str, &'static str, &'static str)] {
    &[
        (
            "effects",
            "Open the effects studio",
            "Palette, duotone and other effects for the selected wallpaper.",
        ),
        ("playlists", "Open playlists", "Make and play lists of wallpapers."),
        ("mixer", "Open the audio mixer", "Volume of every display's wallpaper."),
        ("schedule", "Open the schedule editor", "Wallpapers by time of day."),
        ("theme-designer", "Open the theme designer", "Design your own colour theme."),
        (
            "bench",
            "Run the benchmark",
            "Moves the picker by itself for about 35 seconds and writes a report.",
        ),
        (
            "random",
            "Apply a random wallpaper",
            "Pick any wallpaper from your library and set it now.",
        ),
        ("tag-cloud", "Open the tag panel", "Search wallpapers by tags or a description."),
        ("downloads", "Open downloads", "Find new wallpapers online."),
        ("displays", "Choose displays", "Pick which screens the selected wallpaper goes to."),
        ("home", "Go to Home", "Pinned cards and recent changes."),
        ("edit-cards", "Edit Home cards", "Add, remove and reorder the cards on Home."),
        ("simple-view", "Switch Simple / Advanced view", "Show or hide the advanced settings."),
        ("undo", "Undo the last change", "Put the last changed setting back."),
        ("close", "Close Settings", "Back to the picker."),
    ]
}

fn value_of(control: &Control) -> PaletteValue {
    match control {
        Control::Toggle { path, value } => {
            PaletteValue::Toggle { path: path.clone(), value: *value }
        }
        Control::Chips { path, options, current, disabled, .. } if options.len() <= 12 => {
            PaletteValue::Choice {
                path: path.clone(),
                options: options
                    .iter()
                    .filter(|(key, _)| !disabled.contains(key))
                    .cloned()
                    .collect(),
                current: current.clone(),
            }
        }
        Control::Dropdown { path, options, current, .. } if options.len() <= 12 => {
            PaletteValue::Choice {
                path: path.clone(),
                options: options.clone(),
                current: current.clone(),
            }
        }
        _ => PaletteValue::None,
    }
}

fn haystack(parts: &[&str]) -> String {
    parts.join(" ").to_lowercase()
}

fn push_rows(
    out: &mut Vec<PaletteEntry>,
    page: usize,
    crumb: &str,
    rows: &[Row],
    parent: Option<usize>,
) {
    let mut segment = String::new();
    for (index, row) in rows.iter().enumerate() {
        let target = parent.unwrap_or(index);
        if matches!(row.control, Control::Segment) {
            segment.clone_from(&row.title);
            out.push(PaletteEntry {
                id: format!("{}#{target}", PAGES[page].key),
                kind: PaletteKind::Page,
                icon: PAGES[page].icon,
                title: row.title.clone(),
                crumb: crumb.to_string(),
                desc: row.desc.clone(),
                value: PaletteValue::None,
                action: PaletteAction::Setting { page, row: target },
                haystack: haystack(&[&row.title, crumb, &row.desc]),
            });
            continue;
        }
        if matches!(row.control, Control::Preview) {
            continue;
        }
        let where_ =
            if segment.is_empty() { crumb.to_string() } else { format!("{crumb} › {segment}") };
        let title = if row.title.trim().is_empty() {
            PAGES[page].label.to_string()
        } else {
            row.title.clone()
        };
        let path = control_path(&row.control).unwrap_or_default();
        out.push(PaletteEntry {
            id: format!(
                "{}#{target}:{}",
                PAGES[page].key,
                if path.is_empty() { title.clone() } else { path.clone() }
            ),
            kind: PaletteKind::Setting,
            icon: PAGES[page].icon,
            title: title.clone(),
            crumb: where_.clone(),
            desc: row.desc.clone(),
            value: value_of(&row.control),
            action: PaletteAction::Setting { page, row: target },
            haystack: haystack(&[&title, &where_, &row.desc, &path, &options_text(&row.control)]),
        });
        if let Control::Details { rows: children, .. } | Control::StackBar { rows: children, .. } =
            &row.control
        {
            push_rows(out, page, &format!("{where_} › {title}"), children, Some(target));
        }
    }
}

fn options_text(control: &Control) -> String {
    match control {
        Control::Chips { options, .. } | Control::Dropdown { options, .. } => {
            options.iter().map(|(_, label)| label.as_str()).collect::<Vec<_>>().join(" ")
        }
        Control::ActionChips { items } => {
            items.iter().map(|(_, label)| label.as_str()).collect::<Vec<_>>().join(" ")
        }
        Control::ActionBtn { label, .. } => label.clone(),
        _ => String::new(),
    }
}

/// The config path a row edits, if it edits one.
pub(in crate::frontend::settings) fn control_path(control: &Control) -> Option<String> {
    match control {
        Control::Toggle { path, .. }
        | Control::Number { path, .. }
        | Control::TextField { path, .. }
        | Control::Resolution { path, .. }
        | Control::KeyBinding { path, .. }
        | Control::Dropdown { path, .. }
        | Control::Chips { path, .. } => Some(path.clone()),
        _ => None,
    }
}

/// Crumb of a page: "Group › Page".
pub fn page_crumb(page: usize) -> String {
    format!("{} › {}", GROUPS[PAGES[page].group].label, PAGES[page].label)
}

/// Every palette entry. `page_rows(index)` returns page `PAGES[index]`'s
/// full row list (`PageContent::rows(false)`).
pub fn palette_index(
    page_rows: &mut dyn FnMut(usize) -> Vec<Row>,
    layouts: &[(&'static str, &'static str)],
) -> Vec<PaletteEntry> {
    let mut out = Vec::new();
    for (index, page) in PAGES.iter().enumerate() {
        let crumb = page_crumb(index);
        out.push(PaletteEntry {
            id: page.key.to_string(),
            kind: PaletteKind::Page,
            icon: page.icon,
            title: page.label.to_string(),
            crumb: GROUPS[page.group].label.to_string(),
            desc: page.sentence.to_string(),
            value: PaletteValue::None,
            action: PaletteAction::Page(index),
            haystack: haystack(&[page.label, GROUPS[page.group].label, page.sentence]),
        });
        let rows = page_rows(index);
        push_rows(&mut out, index, &crumb, &rows, None);
    }
    for spec in &transitions::TRANSITIONS {
        if matches!(spec.key, "playlist" | "combo") {
            continue;
        }
        let label = transitions::label_of(spec.key);
        out.push(PaletteEntry {
            id: format!("transition:{}", spec.key),
            kind: PaletteKind::Transition,
            icon: ICON_MOTION,
            title: label.clone(),
            crumb: format!("Motion › Transitions › {}", spec.family.label()),
            desc: spec.desc.to_string(),
            value: PaletteValue::None,
            action: PaletteAction::Transition(spec.key),
            haystack: haystack(&[&label, spec.key, "transition", spec.family.label(), spec.desc]),
        });
    }
    for (key, label) in layouts {
        out.push(PaletteEntry {
            id: format!("layout:{key}"),
            kind: PaletteKind::Layout,
            icon: ICON_LAYOUT,
            title: format!("{label} layout"),
            crumb: String::from("Look › Layout"),
            desc: String::from("Switch the picker to this layout."),
            value: PaletteValue::None,
            action: PaletteAction::Layout(key),
            haystack: haystack(&[label, key, "layout mode view"]),
        });
    }
    for (id, title, desc) in palette_actions() {
        out.push(PaletteEntry {
            id: format!("action:{id}"),
            kind: PaletteKind::Action,
            icon: ICON_ACTION,
            title: (*title).to_string(),
            crumb: String::from("Action"),
            desc: (*desc).to_string(),
            value: PaletteValue::None,
            action: PaletteAction::Run(id),
            haystack: haystack(&[title, desc, id]),
        });
    }
    out
}

/// Fuzzy score of `query` (lower case, one term) in `text` (lower case):
/// every query letter in order; bonuses for word starts and runs. `None`
/// when it does not match.
pub fn fuzzy_score(query: &str, text: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }
    if let Some(at) = text.find(query) {
        let word_start = at == 0 || !text.as_bytes()[at - 1].is_ascii_alphanumeric();
        return Some(1000 - at.min(200) as i32 + if word_start { 300 } else { 0 });
    }
    let mut score = 0;
    let mut chars = text.char_indices().peekable();
    let mut previous: Option<usize> = None;
    let mut last_char = ' ';
    for wanted in query.chars() {
        let mut found = None;
        for (at, character) in chars.by_ref() {
            let before = last_char;
            last_char = character;
            if character == wanted {
                found = Some((at, before));
                break;
            }
        }
        let (at, before) = found?;
        score += 10;
        if !before.is_alphanumeric() {
            score += 25;
        }
        if previous.is_some_and(|previous| at == previous + wanted.len_utf8().max(1)) {
            score += 15;
        }
        previous = Some(at);
    }
    let gaps = previous.unwrap_or(0) as i32 - query.len() as i32;
    Some(score - gaps.max(0) / 4)
}

/// Results for `query`, best first, at most `limit`.
pub fn palette_search<'a>(
    entries: &'a [PaletteEntry],
    query: &str,
    limit: usize,
) -> Vec<&'a PaletteEntry> {
    let query = query.trim().to_lowercase();
    let terms: Vec<&str> = query.split_whitespace().collect();
    if terms.is_empty() {
        return Vec::new();
    }
    let mut scored: Vec<(i32, usize, &PaletteEntry)> = entries
        .iter()
        .enumerate()
        .filter_map(|(order, entry)| {
            let title = entry.title.to_lowercase();
            let mut total = 0;
            for term in &terms {
                let in_title = fuzzy_score(term, &title).map(|score| score * 2);
                let anywhere = fuzzy_score(term, &entry.haystack);
                total += in_title.max(anywhere)?;
            }
            if title == query {
                total += 5000;
            }
            total += match entry.kind {
                PaletteKind::Page => 120,
                PaletteKind::Layout | PaletteKind::Transition => 80,
                PaletteKind::Setting => 60,
                PaletteKind::Action => 40,
            };
            Some((total, order, entry))
        })
        .collect();
    scored.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    scored.into_iter().take(limit).map(|(_, _, entry)| entry).collect()
}
