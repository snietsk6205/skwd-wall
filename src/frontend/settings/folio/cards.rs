//! R1b (custom build): cards. A grid of numbered cards with thin outlines:
//! the number and name on top, a "+" in the corner, the picture inside, and
//! small tags at the bottom. The card under the pointer or the keyboard
//! lights up, its picture plays and its actions replace the tags. Above the
//! grid: a search field and filter words on one line.

use std::collections::HashMap;

use iced::advanced::image::Handle;
use iced::widget::{button, column, container, image, mouse_area, text, text_input};
use iced::{Alignment, Element, Length, Padding};

use crate::app::Message;
use crate::frontend::components::with_alpha;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{
    MONO_FONT, NERD_FONT, TEXT_FONT, TEXT_STRONG_FONT, legible_type_scale, row, typed,
};

use super::super::nav::{ICON_CHECK, ICON_PLUS};
use super::super::{ActionId, Row, SettingsMsg, Tile};

/// Card sizes: the width a card aims for.
pub(super) const CARD_TARGET_WIDTH: f32 = 250.0;
const GAP: f32 = 14.0;

/// Width of a page with cards (and of Home) in a window `panel_width` wide.
pub fn wide_page_width(panel_width: f32, scale: f32) -> f32 {
    (panel_width - 112.0 * scale).clamp(320.0 * scale, 1180.0 * scale)
}

/// Height of one card in a grid `width` wide (for scrolling to a card).
pub fn card_height(width: f32, scale: f32) -> f32 {
    let gap = GAP * scale;
    let columns = columns_for(width, scale) as f32;
    let card = ((width - gap * (columns - 1.0)) / columns).floor();
    ((card - 22.0 * scale) * 9.0 / 16.0).floor() + 84.0 * scale + gap
}

/// How many cards fit in a row of `width`.
pub fn columns_for(width: f32, scale: f32) -> usize {
    let gap = GAP * scale;
    (((width + gap) / (CARD_TARGET_WIDTH * scale + gap)).floor() as usize).clamp(2, 6)
}

/// One card of a grid: its row index on the page and its parts.
pub(super) struct TileItem {
    pub index: usize,
    pub title: String,
    pub items: Vec<(ActionId, String)>,
    pub tile: Tile,
}

impl TileItem {
    pub(super) fn from_row(index: usize, row: Row) -> Option<Self> {
        match row.control {
            super::super::Control::Tile { items, tile } => {
                Some(Self { index, title: row.title, items, tile })
            }
            _ => None,
        }
    }
}

/// What the cards need to draw themselves.
#[derive(Clone, Copy)]
pub(super) struct TileCtx<'a> {
    pub palette: &'a Palette,
    pub scale: f32,
    pub fade: f32,
    pub width: f32,
    pub pictures: &'a HashMap<String, Handle>,
    pub playing: Option<(&'a str, &'a Handle)>,
    /// Row under the pointer and how far its glow is (0..1).
    pub hover: Option<usize>,
    pub hover_fade: f32,
    /// Row the keyboard is on, and the action it is on.
    pub focus: Option<usize>,
    pub choice: Option<usize>,
    /// Number of the grid's first card (cards count on across a page).
    pub first_number: usize,
}

fn picture<'a>(item: &TileItem, ctx: TileCtx<'a>, height: f32) -> Element<'a, Message> {
    let TileCtx { palette, scale, fade, .. } = ctx;
    let handle = ctx
        .playing
        .filter(|(name, _)| *name == item.tile.picture)
        .map(|(_, handle)| handle.clone())
        .or_else(|| ctx.pictures.get(&item.tile.picture).cloned());
    let inside: Element<'a, Message> = match handle {
        Some(handle) => image(handle)
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(iced::ContentFit::Cover)
            .opacity(fade)
            .into(),
        None => container(typed(
            "preparing",
            MONO_FONT,
            9.5,
            scale,
            with_alpha(palette.surface_text, 0.26 * fade),
        ))
        .center(Length::Fill)
        .into(),
    };
    container(inside)
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .clip(true)
        .style(move |_| crate::frontend::ui::ink_fill(palette.surface_variant, 0.7 * fade))
        .into()
}

fn tags<'a>(item: &TileItem, ctx: TileCtx<'a>, height: f32) -> Element<'a, Message> {
    let TileCtx { palette, scale, fade, .. } = ctx;
    let mut line = row![].spacing(5.0 * scale).align_y(Alignment::Center);
    for (name, strong) in item.tile.tags.iter().take(3) {
        let strong = *strong;
        line = line.push(
            container(typed(
                name.clone(),
                MONO_FONT,
                9.5,
                scale,
                with_alpha(
                    if strong { palette.primary } else { palette.surface_text },
                    if strong { fade } else { 0.56 * fade },
                ),
            ))
            .padding([3.0 * scale, 7.0 * scale])
            .style(move |_| crate::frontend::ui::tag_style(palette, fade, strong)),
        );
    }
    container(line).height(Length::Fixed(height)).align_y(Alignment::Center).into()
}

fn actions<'a>(item: &TileItem, ctx: TileCtx<'a>, height: f32) -> Element<'a, Message> {
    let TileCtx { palette, scale, fade, .. } = ctx;
    let plus = item.tile.plus.map(|(index, _)| index);
    let mut line = row![].spacing(5.0 * scale).align_y(Alignment::Center);
    for (index, (id, name)) in item.items.iter().enumerate() {
        if Some(index) == plus {
            continue;
        }
        let primary = index == item.tile.primary;
        let chosen = ctx.focus == Some(item.index) && ctx.choice == Some(index);
        line = line.push(
            button(
                container(text(name.clone()).font(MONO_FONT).size(9.5 * legible_type_scale(scale)))
                    .center_x(Length::Fill),
            )
            .width(Length::FillPortion(if name.chars().count() > 6 { 4 } else { 3 }))
            .padding([4.0 * scale, 4.0 * scale])
            .on_press(Message::Settings(SettingsMsg::Run(id.clone())))
            .style(move |_theme, status| {
                crate::frontend::ui::tile_action_style(palette, fade, primary, chosen, status)
            }),
        );
    }
    container(line).height(Length::Fixed(height)).align_y(Alignment::Center).into()
}

fn card<'a>(item: &TileItem, number: usize, width: f32, ctx: TileCtx<'a>) -> Element<'a, Message> {
    let TileCtx { palette, scale, fade, .. } = ctx;
    let hovered = ctx.hover == Some(item.index);
    let focused = ctx.focus == Some(item.index);
    let lit = hovered || focused;
    let glow = if hovered { ctx.hover_fade.clamp(0.0, 1.0) } else { 0.0 };
    let pad = 11.0 * scale;
    let picture_height = ((width - 2.0 * pad) * 9.0 / 16.0).floor();
    let footer_height = 24.0 * scale;
    let active = item.tile.active;
    let number = typed(
        format!("{number:02}"),
        MONO_FONT,
        10.0,
        scale,
        with_alpha(
            if active { palette.primary } else { palette.surface_text },
            if active { fade } else { 0.36 * fade },
        ),
    );
    let name = typed(
        item.title.clone(),
        TEXT_STRONG_FONT,
        13.0,
        scale,
        with_alpha(palette.surface_text, if lit || active { fade } else { 0.86 * fade }),
    )
    .wrapping(iced::widget::text::Wrapping::None);
    let plus: Element<'a, Message> = match item.tile.plus {
        Some((index, on)) => {
            let chosen = focused && ctx.choice == Some(index);
            let message = item
                .items
                .get(index)
                .map_or(Message::Noop, |(id, _)| Message::Settings(SettingsMsg::Run(id.clone())));
            button(
                text(if on { ICON_CHECK } else { ICON_PLUS })
                    .font(NERD_FONT)
                    .size(12.0 * legible_type_scale(scale))
                    .color(with_alpha(
                        if on { palette.primary } else { palette.surface_text },
                        if on || lit { fade } else { 0.5 * fade },
                    )),
            )
            .padding([1.0 * scale, 4.0 * scale])
            .on_press(message)
            .style(move |_theme, status| {
                crate::frontend::ui::text_button_style(palette, fade, on, chosen, status)
            })
            .into()
        }
        None => text("").into(),
    };
    let header = row![
        container(number).padding(Padding { top: 2.0 * scale, ..Padding::ZERO }),
        container(name).width(Length::Fill).clip(true),
        plus,
    ]
    .spacing(8.0 * scale)
    .align_y(Alignment::Center)
    .height(Length::Fixed(20.0 * scale));
    let footer =
        if lit { actions(item, ctx, footer_height) } else { tags(item, ctx, footer_height) };
    let index = item.index;
    let body = column![header, picture(item, ctx, picture_height), footer].spacing(9.0 * scale);
    mouse_area(
        container(body)
            .width(Length::Fixed(width))
            .padding(pad)
            .style(move |_| crate::frontend::ui::tile_style(palette, fade, glow, focused, false)),
    )
    .on_enter(Message::Settings(SettingsMsg::TileHover(index, true)))
    .on_exit(Message::Settings(SettingsMsg::TileHover(index, false)))
    .on_press(Message::Settings(SettingsMsg::FocusControl(index)))
    .into()
}

/// A grid of cards that fits the width.
pub(super) fn tile_grid(items: Vec<TileItem>, ctx: TileCtx<'_>) -> Element<'_, Message> {
    let gap = GAP * ctx.scale;
    let columns = columns_for(ctx.width, ctx.scale);
    let width = ((ctx.width - gap * (columns as f32 - 1.0)) / columns as f32).floor();
    let mut grid = column![].spacing(gap);
    let mut line = row![].spacing(gap);
    let mut in_line = 0;
    for (position, item) in items.into_iter().enumerate() {
        line = line.push(card(&item, ctx.first_number + position, width, ctx));
        in_line += 1;
        if in_line == columns {
            grid = grid.push(std::mem::replace(&mut line, row![].spacing(gap)));
            in_line = 0;
        }
    }
    if in_line > 0 {
        grid = grid.push(line);
    }
    grid.into()
}

/// What the line above a grid shows.
pub(super) struct Toolbar<'a> {
    /// The search field: its key and placeholder.
    pub search: Option<(String, &'static str)>,
    pub values: &'a HashMap<String, String>,
    /// The filter: its path, choices and the chosen one.
    pub filter: Option<(String, Vec<(String, String)>, String)>,
    /// The keyboard is on the filter, and the choice it is on.
    pub filter_focused: bool,
    pub filter_choice: Option<usize>,
    pub search_focus: bool,
    /// A short note at the end (what the lit card does, or a count).
    pub note: String,
}

/// Search and filter words on one line, with a quiet note at the end.
pub(super) fn toolbar<'a>(input: Toolbar<'a>, ctx: TileCtx<'a>) -> Element<'a, Message> {
    let TileCtx { palette, scale, fade, .. } = ctx;
    let mut line = row![].spacing(4.0 * scale).align_y(Alignment::Center);
    if let Some((key, placeholder)) = input.search {
        let value = input.values.get(&key).map_or("", String::as_str);
        let message_key = key.clone();
        let focused = input.search_focus;
        line = line.push(
            container(
                text_input(placeholder, value)
                    .id(super::super::workbench_input_id(&key))
                    .font(TEXT_FONT)
                    .size(12.0 * legible_type_scale(scale))
                    .padding([6.0 * scale, 10.0 * scale])
                    .width(Length::Fixed(200.0 * scale))
                    .on_input(move |raw| {
                        Message::Settings(SettingsMsg::Input(message_key.clone(), raw))
                    })
                    .on_submit(Message::Settings(SettingsMsg::Commit))
                    .style(move |_theme, status| {
                        let status = if focused { text_input::Status::Hovered } else { status };
                        crate::frontend::ui::quiet_input_style(palette, fade, status)
                    }),
            )
            .padding(Padding { right: 10.0 * scale, ..Padding::ZERO }),
        );
    }
    if let Some((path, options, current)) = input.filter {
        for (position, (key, name)) in options.into_iter().enumerate() {
            let chosen = key == current;
            let keyboard = input.filter_focused && input.filter_choice == Some(position);
            line = line.push(
                button(text(name).font(MONO_FONT).size(10.5 * legible_type_scale(scale)))
                    .padding([4.0 * scale, 8.0 * scale])
                    .on_press(Message::Settings(SettingsMsg::Pick(path.clone(), key)))
                    .style(move |_theme, status| {
                        let status = if keyboard { button::Status::Hovered } else { status };
                        crate::frontend::ui::filter_button_style(palette, fade, chosen, status)
                    }),
            );
        }
    }
    let note =
        typed(input.note, TEXT_FONT, 11.5, scale, with_alpha(palette.surface_text, 0.5 * fade))
            .wrapping(iced::widget::text::Wrapping::None);
    column![
        line.wrap(),
        container(note).height(Length::Fixed(16.0 * scale)).clip(true).width(Length::Fill),
    ]
    .spacing(8.0 * scale)
    .into()
}

mod tests;
