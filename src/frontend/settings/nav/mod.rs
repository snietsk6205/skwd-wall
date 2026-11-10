//! R1 (custom build): the Settings top bar's six groups, their pages and the
//! old tab/section each page shows. English only, like the other custom cards.

mod advanced;
mod content;
mod explain;
mod gallery;
mod icons;
mod map;
mod palette;

pub use advanced::{ADVANCED_PAGES, row_is_advanced};
pub use content::{locate, page_content, page_for_section, page_has_cards, simple_rows};
pub use explain::more_text;
pub use gallery::{CardPicture, SKETCH_STRIP_PREFIX, gallery};
pub use icons::*;
pub use map::{GROUPS, PAGES, find_page, group_pages, page_index, static_tab};
pub use palette::{PaletteAction, PaletteEntry, PaletteValue, palette_index, palette_search};

#[cfg(test)]
mod tests;
