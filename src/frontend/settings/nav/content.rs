use std::collections::HashMap;

use super::super::{Card, Control, Row};
use super::map::{PAGES, Page, Ref};

/// The first old section a page shows. Old code that keys on the tab and
/// section (folders, profiles, layout studio) keeps working with it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PagePrimary {
    pub tab: &'static str,
    pub section: usize,
}

/// The old sections one page shows, in the page's order.
pub struct PageContent {
    pub cards: Vec<(Card, Vec<Row>)>,
    pub sections: Vec<(&'static str, usize)>,
}

impl PageContent {
    pub fn primary(&self) -> Option<PagePrimary> {
        self.sections.first().map(|&(tab, section)| PagePrimary { tab, section })
    }

    /// Every row of the page as one list. Several sections get a heading
    /// row (`Control::Segment`) each. `simple` hides the advanced rows (and
    /// headings left without rows).
    pub fn rows(self, simple: bool) -> Vec<Row> {
        let several = self.cards.len() > 1;
        let mut out = Vec::new();
        for (card, rows) in self.cards {
            let mut part: Vec<Row> = Vec::new();
            if several {
                part.push(Row {
                    title: card.title.to_string(),
                    desc: card.subtitle.to_string(),
                    control: Control::Segment,
                });
            }
            part.extend(rows);
            out.extend(part);
        }
        if simple { simple_rows(out) } else { out }
    }

    /// Row index inside `rows(false)` of row `row` of old section `section`.
    pub fn row_offset(&self, tab: &str, section: usize, row: usize) -> Option<usize> {
        let several = self.cards.len() > 1;
        let mut offset = 0;
        for ((card_tab, card_section), (_, rows)) in self.sections.iter().zip(&self.cards) {
            if several {
                offset += 1;
            }
            if *card_tab == tab && *card_section == section {
                return Some(offset + row.min(rows.len().saturating_sub(1)));
            }
            offset += rows.len();
        }
        None
    }
}

/// What Simple view keeps of a page's full row list.
pub fn simple_rows(rows: Vec<Row>) -> Vec<Row> {
    let kept: Vec<Row> =
        rows.into_iter().filter(|row| !super::advanced::row_is_advanced(row)).collect();
    let mut out: Vec<Row> = Vec::with_capacity(kept.len());
    for row in kept {
        if matches!(row.control, Control::Segment)
            && out.last().is_some_and(|last| matches!(last.control, Control::Segment))
        {
            out.pop();
        }
        out.push(row);
    }
    if out.last().is_some_and(|last| matches!(last.control, Control::Segment)) {
        out.pop();
    }
    out
}

/// Titles every page names in this tab (what `Ref::Rest` leaves out).
fn claimed(tab: &str) -> Vec<&'static str> {
    PAGES
        .iter()
        .flat_map(|page| page.refs.iter())
        .filter_map(|reference| match reference {
            Ref::Section(owner, title) if *owner == tab => Some(title.text()),
            _ => None,
        })
        .collect()
}

fn matching(reference: Ref, cards: &[(Card, Vec<Row>)]) -> Vec<usize> {
    match reference {
        Ref::Section(_, title) => {
            let title = title.text();
            cards
                .iter()
                .enumerate()
                .filter(|(_, (card, _))| card.title == title)
                .map(|(index, _)| index)
                .collect()
        }
        Ref::Rest(tab) => {
            let claimed = claimed(tab);
            cards
                .iter()
                .enumerate()
                .filter(|(_, (card, _))| !claimed.contains(&card.title))
                .map(|(index, _)| index)
                .collect()
        }
    }
}

type Build<'b> = dyn FnMut(&str) -> Vec<(Card, Vec<Row>)> + 'b;

struct Tabs<'a, 'b> {
    build: &'a mut Build<'b>,
    cache: HashMap<&'static str, Vec<(Card, Vec<Row>)>>,
}

impl Tabs<'_, '_> {
    fn get(&mut self, tab: &'static str) -> &Vec<(Card, Vec<Row>)> {
        let build = &mut self.build;
        self.cache.entry(tab).or_insert_with(|| build(tab))
    }
}

/// The sections `page` shows. `build` returns one old tab's sections.
pub fn page_content(page: &Page, build: &mut Build<'_>) -> PageContent {
    let mut tabs = Tabs { build, cache: HashMap::new() };
    let mut picked: Vec<(&'static str, usize)> = Vec::new();
    for reference in page.refs {
        let tab = reference.tab();
        for index in matching(*reference, tabs.get(tab)) {
            if !picked.contains(&(tab, index)) {
                picked.push((tab, index));
            }
        }
    }
    let mut cards = Vec::with_capacity(picked.len());
    for &(tab, index) in &picked {
        let built = tabs.cache.get_mut(tab).expect("tab built above");
        let (card, rows) = &mut built[index];
        cards.push((card.clone(), std::mem::take(rows)));
    }
    PageContent { cards, sections: picked }
}

/// Whether `page` has anything to show with the current settings.
pub fn page_has_cards(page: &Page, build: &mut Build<'_>) -> bool {
    let mut tabs = Tabs { build, cache: HashMap::new() };
    page.refs.iter().any(|reference| !matching(*reference, tabs.get(reference.tab())).is_empty())
}

/// The page that shows old section `section` of `tab`.
pub fn page_for_section(tab: &str, section: usize, build: &mut Build<'_>) -> Option<usize> {
    let cards = build(tab);
    PAGES.iter().position(|page| {
        page.refs
            .iter()
            .filter(|reference| reference.tab() == tab)
            .any(|reference| matching(*reference, &cards).contains(&section))
    })
}

/// The page and row (in the page's full row list) of row `row` of old
/// section `section` of `tab`.
pub fn locate(
    tab: &str,
    section: usize,
    row: usize,
    build: &mut Build<'_>,
) -> Option<(usize, usize)> {
    let index = page_for_section(tab, section, build)?;
    let content = page_content(&PAGES[index], build);
    Some((index, content.row_offset(tab, section, row)?))
}
