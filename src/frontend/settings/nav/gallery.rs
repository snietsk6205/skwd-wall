//! R1b: which choices show as a grid of cards instead of a line of words,
//! and what picture each card has.

use crate::contracts::settings::{SettingsSource, keys, transitions};

use super::super::Control;
use super::icons::ICON_BAR;

/// Strip names of the sketch styles start with this.
pub const SKETCH_STRIP_PREFIX: &str = "sketch:";

/// What a card shows inside.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardPicture {
    /// A frame strip (the name the strip cache knows it by).
    Strip(String),
    /// A row of colours (hex).
    Swatches(Vec<String>),
    /// One big icon.
    Glyph(&'static str),
}

/// A row whose choices show as cards.
pub struct Gallery<'a> {
    pub path: &'a str,
    pub options: &'a [(String, String)],
    pub current: &'a str,
    pub disabled: &'a [String],
    palettes: &'a [(String, Vec<String>)],
}

/// The choices of `control` as cards, if this setting shows as cards.
pub fn gallery(control: &Control) -> Option<Gallery<'_>> {
    match control {
        Control::Chips { path, options, current, disabled, .. }
            if matches!(
                path.as_str(),
                keys::transition::SKETCH_STYLE
                    | keys::transition::COMBO_MODE
                    | keys::transition::COMBO_SPLIT
                    | keys::filter_bar::VISUAL_STYLE
            ) =>
        {
            Some(Gallery { path, options, current, disabled, palettes: &[] })
        }
        // Theme presets carry their colours.
        Control::Dropdown { path, options, current, palettes } if !palettes.is_empty() => {
            Some(Gallery { path, options, current, disabled: &[], palettes })
        }
        _ => None,
    }
}

impl Gallery<'_> {
    /// The picture of the card for choice `key`.
    pub fn picture(&self, key: &str, cfg: &dyn SettingsSource) -> CardPicture {
        let text = |path: &str, fallback: &str| {
            let value = cfg.text(path);
            if value.is_empty() { fallback.to_string() } else { value }
        };
        let combo = |mode: &str, split: &str| {
            CardPicture::Strip(transitions::combo_name(
                mode,
                split,
                &text(keys::transition::COMBO_FIRST, "burn"),
                &text(keys::transition::COMBO_SECOND, "ripple"),
            ))
        };
        match self.path {
            keys::transition::SKETCH_STYLE => {
                CardPicture::Strip(format!("{SKETCH_STRIP_PREFIX}{key}"))
            }
            keys::transition::COMBO_MODE => combo(key, &text(keys::transition::COMBO_SPLIT, "lr")),
            keys::transition::COMBO_SPLIT => combo("split", key),
            keys::filter_bar::VISUAL_STYLE => CardPicture::Glyph(ICON_BAR),
            _ => self
                .palettes
                .iter()
                .rev()
                .find(|(name, _)| name == key)
                .map_or(CardPicture::Glyph(super::icons::ICON_COLOURS), |(_, colours)| {
                    CardPicture::Swatches(colours.clone())
                }),
        }
    }

    /// Whether "Preview big" can play this choice (combinations can).
    pub fn previews(&self) -> bool {
        matches!(self.path, keys::transition::COMBO_MODE | keys::transition::COMBO_SPLIT)
    }
}
