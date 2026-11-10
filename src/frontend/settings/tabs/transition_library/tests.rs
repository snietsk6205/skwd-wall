#![cfg(test)]

use super::*;
use crate::frontend::settings::test_source::FakeSettingsSource;

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_string()).collect()
}

fn cards(cfg: &FakeSettingsSource) -> Vec<(super::super::Card, Vec<super::super::Row>)> {
    let mut builder = Builder { cfg, cards: Vec::new() };
    tab_library(&mut builder);
    tab_playlist(&mut builder);
    tab_combo(&mut builder);
    builder.cards
}

fn titles(rows: &[super::super::Row]) -> Vec<String> {
    rows.iter().map(|row| row.title.clone()).collect()
}

#[test]
fn lists_toggle_and_move() {
    let list = strings(&["burn", "ripple"]);
    assert_eq!(list_toggled(&list, "iris"), strings(&["burn", "ripple", "iris"]));
    assert_eq!(list_toggled(&list, "burn"), strings(&["ripple"]));
    assert_eq!(list_moved(&list, 1, true), strings(&["ripple", "burn"]));
    assert_eq!(list_moved(&list, 0, true), list);
    assert_eq!(list_moved(&list, 1, false), list);
    assert_eq!(transition_list(" burn, ,ripple "), list);
}

#[test]
fn library_lists_every_real_transition_once() {
    let cfg = FakeSettingsSource::default();
    let built = cards(&cfg);
    let (_, rows) = built.iter().find(|(card, _)| card.title == LIBRARY_CARD).unwrap();
    let names = titles(rows);
    let expected = transitions::TRANSITIONS
        .iter()
        .filter(|spec| !matches!(spec.key, "random" | "playlist" | "combo"))
        .count();
    let listed = rows.iter().filter(|row| matches!(row.control, Control::Tile { .. })).count();
    assert_eq!(listed, expected, "{names:?}");
    assert!(
        names.contains(&String::from("Light Leak")) && names.contains(&String::from("Random Mix"))
    );
    assert!(!names.contains(&String::from("Playlist")));
}

#[test]
fn library_search_and_filters() {
    let favourites = strings(&["burn"]);
    let playlist = strings(&["ripple"]);
    let burn = transitions::spec("burn").unwrap();
    let ripple = transitions::spec("ripple").unwrap();
    let aurora = transitions::spec("aurora").unwrap();
    assert!(library_matches(burn, "", "all", &favourites, &playlist));
    assert!(library_matches(burn, "", "favourites", &favourites, &playlist));
    assert!(!library_matches(ripple, "", "favourites", &favourites, &playlist));
    assert!(library_matches(ripple, "", "playlist", &favourites, &playlist));
    assert!(library_matches(aurora, "", "light", &favourites, &playlist));
    assert!(!library_matches(burn, "", "light", &favourites, &playlist));
    assert!(library_matches(ripple, "WATER rings", "all", &favourites, &playlist));
    assert!(library_matches(aurora, "northern", "all", &favourites, &playlist));
    assert!(!library_matches(aurora, "water", "all", &favourites, &playlist));

    let cfg = FakeSettingsSource::default()
        .with_text(keys::transition::LIBRARY_SEARCH, "zzz-nothing")
        .with_text(keys::transition::LIBRARY_FILTER, "favourites");
    let built = cards(&cfg);
    let (_, rows) = built.iter().find(|(card, _)| card.title == LIBRARY_CARD).unwrap();
    assert!(titles(rows).contains(&String::from("No transitions")));
}

#[test]
fn playlist_rows_follow_the_list() {
    let cfg = FakeSettingsSource::default()
        .with_text(keys::transition::PLAYLIST, "burn,combo:then:iris:glitch")
        .with_text(keys::transition::PLAYLIST_ORDER, "no-repeat");
    let built = cards(&cfg);
    let (_, rows) = built.iter().find(|(card, _)| card.title == PLAYLIST_CARD).unwrap();
    let names = titles(rows);
    assert!(names.contains(&String::from("1. Burn")), "{names:?}");
    assert!(names.contains(&String::from("2. Iris then Glitch")), "{names:?}");
    assert!(names.contains(&String::from("Use the playlist")));
    let order = rows.iter().find_map(|row| match &row.control {
        Control::Chips { path, current, .. } if path == keys::transition::PLAYLIST_ORDER => {
            Some(current.clone())
        }
        _ => None,
    });
    assert_eq!(order.as_deref(), Some("no-repeat"));
}

#[test]
fn combo_card_builds_the_renderer_name() {
    let cfg = FakeSettingsSource::default()
        .with_text(keys::transition::COMBO_FIRST, "iris")
        .with_text(keys::transition::COMBO_SECOND, "glitch")
        .with_text(keys::transition::COMBO_MODE, "split")
        .with_text(keys::transition::COMBO_SPLIT, "circle");
    assert_eq!(transition_combo_name(&cfg), "combo:split-circle:iris:glitch");
    let built = cards(&cfg);
    let (_, rows) = built.iter().find(|(card, _)| card.title == COMBO_CARD).unwrap();
    assert!(titles(rows).contains(&String::from("Split shape")));
    assert!(titles(rows).contains(&String::from("First transition: family")));
    let first = rows.iter().find_map(|row| match &row.control {
        Control::Chips { path, options, current, .. } if path == keys::transition::COMBO_FIRST => {
            Some((options.len(), current.clone()))
        }
        _ => None,
    });
    let (count, current) = first.unwrap();
    assert_eq!(current, "iris");
    assert!(count < 20, "one family only, got {count}");
    assert_eq!(combo_part_family("iris"), "wipe");
    assert_eq!(combo_part_family("random"), "random");
    assert!(titles(rows).contains(&String::from("Iris | Glitch")));
    let defaults = FakeSettingsSource::default();
    assert_eq!(transition_combo_name(&defaults), "combo:then:burn:ripple");
}
