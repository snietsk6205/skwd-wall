use std::collections::BTreeMap;

use super::palette::{PaletteKind, fuzzy_score};
use super::*;
use crate::frontend::settings::tables::TABS;
use crate::frontend::settings::test_source::FakeSettingsSource;
use crate::frontend::settings::{Card, Row, build_tab};

fn builder(cfg: &FakeSettingsSource) -> impl FnMut(&str) -> Vec<(Card, Vec<Row>)> + '_ {
    move |tab| build_tab(tab, cfg, &[], &[], "", &[])
}

#[test]
fn every_section_has_exactly_one_page() {
    let cfg = FakeSettingsSource::default();
    let mut build = builder(&cfg);
    let mut owners: BTreeMap<(String, usize), Vec<&str>> = BTreeMap::new();
    for page in &PAGES {
        let content = page_content(page, &mut build);
        for (tab, section) in content.sections {
            owners.entry((tab.to_string(), section)).or_default().push(page.key);
        }
    }
    for tab in TABS {
        for (section, (card, _)) in build_tab(tab, &cfg, &[], &[], "", &[]).iter().enumerate() {
            let pages = owners.get(&(tab.to_string(), section)).cloned().unwrap_or_default();
            assert_eq!(
                pages.len(),
                1,
                "{tab} section {section} ({}) shown by {pages:?}",
                card.title
            );
        }
    }
}

#[test]
fn page_keys_unique_and_grouped() {
    let mut keys: Vec<_> = PAGES.iter().map(|page| page.key).collect();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), PAGES.len());
    for page in &PAGES {
        assert!(page.group >= 1 && page.group < GROUPS.len(), "{}", page.key);
        assert!(page.key.starts_with(GROUPS[page.group].key), "{}", page.key);
        assert!(!page.sentence.is_empty() && page.sentence.ends_with('.'), "{}", page.key);
    }
    for (group, info) in GROUPS.iter().enumerate().skip(1) {
        assert!(group_pages(group).count() >= 4, "{}", info.key);
    }
}

#[test]
fn default_home_cards_are_pages() {
    let default = crate::contracts::settings::schema::text_default(
        crate::contracts::settings::keys::settings_window::HOME_CARDS,
    )
    .unwrap_or_default();
    for key in default.split(',') {
        assert!(page_index(key).is_some(), "{key}");
    }
}

#[test]
fn locate_finds_rows_of_combined_pages() {
    let cfg = FakeSettingsSource::default();
    let mut build = builder(&cfg);
    let bar = page_index("look/bar").expect("bar page");
    let buttons = build("filter")
        .iter()
        .position(|(card, _)| card.title == crate::i18n::tr("settings-section-buttons"))
        .expect("buttons section");
    let (index, row) = locate("filter", buttons, 2, &mut build).expect("located");
    assert_eq!(index, bar);
    let rows = page_content(&PAGES[bar], &mut build).rows(false);
    let expected = &build("filter")[buttons].1[2];
    assert_eq!(rows[row].title, expected.title);
}

#[test]
fn fuzzy_matching_prefers_title_starts() {
    assert!(
        fuzzy_score("trans", "transitions").unwrap()
            > fuzzy_score("trans", "quick transitions").unwrap()
    );
    assert!(fuzzy_score("lpm", "low power mode").is_some());
    assert!(fuzzy_score("xyz", "low power mode").is_none());
}

#[test]
fn palette_finds_settings_transitions_layouts_and_actions() {
    let cfg = FakeSettingsSource::default();
    let mut build = builder(&cfg);
    let mut rows = |index: usize| page_content(&PAGES[index], &mut build).rows(false);
    let entries = palette_index(&mut rows, &[("coverflow", "Coverflow")]);
    let first = |query: &str| palette_search(&entries, query, 5).first().map(|entry| entry.kind);
    assert_eq!(first("low power"), Some(PaletteKind::Setting));
    assert_eq!(first("honeycomb"), Some(PaletteKind::Transition));
    assert_eq!(first("coverflow layout"), Some(PaletteKind::Layout));
    assert_eq!(first("benchmark"), Some(PaletteKind::Action));
    assert!(entries.iter().any(|entry| matches!(entry.value, PaletteValue::Toggle { .. })));
    assert!(palette_search(&entries, "", 5).is_empty());
}

#[test]
fn simple_view_drops_empty_headings() {
    let cfg = FakeSettingsSource::default();
    let mut build = builder(&cfg);
    for page in &PAGES {
        let rows = page_content(page, &mut build).rows(true);
        assert!(
            rows.last().is_none_or(|row| !matches!(
                row.control,
                crate::frontend::settings::Control::Segment
            )),
            "{}",
            page.key
        );
    }
}

#[test]
#[ignore = "dump for the handoff"]
fn dump_pages() {
    let cfg = FakeSettingsSource::default();
    let mut build = builder(&cfg);
    for page in &PAGES {
        println!("== {} ({})", page.key, page.label);
        for row in page_content(page, &mut build).rows(false) {
            let path = palette::control_path(&row.control).unwrap_or_default();
            let kind = format!("{:?}", row.control)
                .split([' ', '{', '('])
                .next()
                .unwrap_or("")
                .to_string();
            println!("   [{kind}] {} | {path}", row.title);
        }
    }
}

#[test]
fn advanced_and_explained_paths_are_in_the_schema() {
    use crate::contracts::settings::schema;
    for path in advanced::advanced_paths() {
        assert!(schema::value_kind(path).is_some(), "advanced {path}");
    }
    for path in explain::explained_paths() {
        assert!(schema::value_kind(path).is_some(), "explained {path}");
    }
}

#[test]
fn simple_view_keeps_most_rows() {
    let cfg = FakeSettingsSource::default();
    let mut build = builder(&cfg);
    let mut full = 0;
    let mut simple = 0;
    for page in &PAGES {
        full += page_content(page, &mut build).rows(false).len();
        simple += page_content(page, &mut build).rows(true).len();
    }
    assert!(simple < full && simple * 10 > full * 6, "{simple} of {full}");
}
