//! R1 (custom build): Home: a living picture behind pinned cards, the
//! latest changes, a pause button for a video background and "Edit cards".

use iced::widget::{button, column, container, mouse_area, text};
use iced::{Alignment, Color, Element, Length, Padding};

use crate::app::Message;
use crate::frontend::components::with_alpha;
use crate::frontend::theme::Palette;
use crate::frontend::ui::pill_button_style as pill_style;
use crate::frontend::ui::{TYPE_SMALL, label, legible_type_scale, row};

use super::super::nav::{self, GROUPS, PAGES};
use super::super::{SettingsFocus, SettingsMsg};
use super::shell::NavView;

fn icon<'a>(glyph: &'static str, size: f32, scale: f32, color: Color) -> iced::widget::Text<'a> {
    text(glyph)
        .font(crate::frontend::ui::NERD_FONT)
        .size(size * legible_type_scale(scale))
        .color(color)
}

fn small_button<'a>(
    glyph: &'static str,
    text_: &'static str,
    message: SettingsMsg,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    button(
        row![
            icon(glyph, 11.0, scale, with_alpha(palette.surface_text, fade)),
            label(text_, 10.5, scale, with_alpha(palette.surface_text, fade)),
        ]
        .spacing(4.0 * scale)
        .align_y(Alignment::Center),
    )
    .padding([3.0 * scale, 7.0 * scale])
    .on_press(Message::Settings(message))
    .style(move |_theme, status| pill_style(palette, fade, status, false))
    .into()
}

#[allow(clippy::too_many_lines)]
pub(super) fn home<'a>(
    nav: &NavView<'a>,
    focus: SettingsFocus,
    width: f32,
    height: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    let columns = 3usize;
    let gap = 14.0 * scale;
    let grid_width = width.min(900.0 * scale);
    let card_width = ((grid_width - gap * (columns as f32 - 1.0)) / columns as f32).floor();
    let card_height = 118.0 * scale;
    let home_focus = focus == SettingsFocus::Home;
    let mut grid = column![].spacing(gap);
    let mut line = row![].spacing(gap);
    let mut in_line = 0;
    let count = nav.home_cards.len();
    for (position, &page_index) in nav.home_cards.iter().enumerate() {
        let page = &PAGES[page_index];
        let hovered = nav.card_hover == Some(position);
        let focused = home_focus && nav.card_focus == position;
        let lift = if hovered {
            crate::frontend::animation::smoothstep(nav.card_fade.clamp(0.0, 1.0))
        } else {
            0.0
        };
        let glyph = container(icon(page.icon, 20.0, scale, with_alpha(palette.primary, fade)))
            .center(Length::Fixed(38.0 * scale))
            .style(move |_| {
                crate::frontend::ui::surface_style(
                    with_alpha(palette.primary, (0.14 + 0.08 * lift) * fade),
                    10.0,
                    (0.0, Color::TRANSPARENT),
                    None,
                )
            });
        let mut body = column![
            row![
                glyph,
                container(text("")).width(Length::Fill),
                label(
                    GROUPS[page.group].label,
                    9.5,
                    scale,
                    with_alpha(palette.surface_text, 0.5 * fade)
                ),
            ]
            .align_y(Alignment::Start),
            label(page.label, 15.0, scale, with_alpha(palette.surface_text, fade)),
            label(page.sentence, TYPE_SMALL, scale, with_alpha(palette.surface_text, 0.66 * fade))
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
        ]
        .spacing(5.0 * scale);
        if nav.edit_cards {
            body = body.push(
                row![
                    small_button(
                        nav::ICON_BACK,
                        "Earlier",
                        SettingsMsg::CardMove(position, true),
                        scale,
                        palette,
                        fade
                    ),
                    small_button(
                        nav::ICON_CHEVRON,
                        "Later",
                        SettingsMsg::CardMove(position, false),
                        scale,
                        palette,
                        fade
                    ),
                    container(text("")).width(Length::Fill),
                    small_button(
                        nav::ICON_CLOSE,
                        "Remove",
                        SettingsMsg::CardRemove(position),
                        scale,
                        palette,
                        fade
                    ),
                ]
                .spacing(4.0 * scale)
                .align_y(Alignment::Center),
            );
        }
        let message = if nav.edit_cards {
            Message::Noop
        } else {
            Message::Settings(SettingsMsg::OpenPage(page_index))
        };
        let card = button(body)
            .width(Length::Fixed(card_width))
            .height(Length::Fixed(card_height + if nav.edit_cards { 26.0 * scale } else { 0.0 }))
            .padding(Padding {
                top: 12.0 * scale - 3.0 * lift * scale,
                right: 14.0 * scale,
                bottom: 12.0 * scale,
                left: 14.0 * scale,
            })
            .on_press(message)
            .style(move |_theme, _status| {
                crate::frontend::ui::rounded_button_style(
                    with_alpha(palette.surface, (0.66 + 0.16 * lift) * fade),
                    14.0,
                    (
                        if focused { 1.6 } else { 1.0 },
                        with_alpha(
                            if focused || hovered { palette.primary } else { palette.outline },
                            if focused { fade } else { (0.42 + 0.4 * lift) * fade },
                        ),
                    ),
                    with_alpha(palette.surface_text, fade),
                    Some((
                        with_alpha(Color::BLACK, (0.18 + 0.2 * lift) * fade),
                        4.0 + 6.0 * lift,
                        14.0 + 12.0 * lift,
                    )),
                )
            });
        let card = mouse_area(card)
            .on_enter(Message::Settings(SettingsMsg::CardHover(Some(position))))
            .on_exit(Message::Settings(SettingsMsg::CardHover(None)));
        line = line.push(card);
        in_line += 1;
        if in_line == columns {
            grid = grid.push(std::mem::replace(&mut line, row![].spacing(gap)));
            in_line = 0;
        }
    }
    if nav.edit_cards {
        let focused = home_focus && nav.card_focus >= count;
        let add = button(
            column![
                icon(nav::ICON_PLUS, 22.0, scale, with_alpha(palette.primary, fade)),
                label("Add a card", 14.0, scale, with_alpha(palette.surface_text, fade)),
                label(
                    "Pick any page from the menus above.",
                    TYPE_SMALL,
                    scale,
                    with_alpha(palette.surface_text, 0.62 * fade)
                ),
            ]
            .spacing(5.0 * scale)
            .align_x(Alignment::Center),
        )
        .width(Length::Fixed(card_width))
        .height(Length::Fixed(card_height + 26.0 * scale))
        .padding(14.0 * scale)
        .on_press(Message::Settings(SettingsMsg::MenuClick(1)))
        .style(move |_theme, status| {
            let hovered = matches!(status, button::Status::Hovered);
            crate::frontend::ui::rounded_button_style(
                with_alpha(palette.surface, if hovered { 0.6 } else { 0.42 } * fade),
                14.0,
                (
                    if focused { 1.6 } else { 1.0 },
                    with_alpha(if focused { palette.primary } else { palette.outline }, 0.7 * fade),
                ),
                Color::TRANSPARENT,
                None,
            )
        });
        line = line.push(add);
        in_line += 1;
    }
    if in_line > 0 {
        grid = grid.push(line);
    }
    let recent = recent_card(
        nav,
        home_focus && !nav.edit_cards && nav.card_focus >= count,
        grid_width,
        scale,
        palette,
        fade,
    );
    let heading = column![
        text(if nav.edit_cards { "Edit your cards" } else { "What would you like to change?" })
            .font(crate::frontend::ui::UI_FONT)
            .size(30.0 * scale.clamp(0.85, 1.1))
            .color(with_alpha(palette.surface_text, fade)),
        label(
            if nav.edit_cards {
                "Earlier and Later move a card, Remove takes it off. With the keyboard: arrows choose, Shift+arrows move, Delete removes, Esc ends."
            } else {
                "Your pinned pages. The menus above hold everything else; Ctrl+K finds any setting."
            },
            12.0,
            scale,
            with_alpha(palette.surface_text, 0.72 * fade),
        ),
    ]
    .spacing(4.0 * scale)
    .width(Length::Fixed(grid_width));
    let editing = nav.edit_cards;
    let mut footer = row![].spacing(8.0 * scale).align_y(Alignment::Center);
    if nav.backdrop.video {
        let (glyph, words) = if nav.backdrop.paused || nav.backdrop.held.is_some() {
            (nav::ICON_PLAY, "Play background")
        } else {
            (nav::ICON_PAUSE_SMALL, "Pause background")
        };
        footer = footer.push(
            button(
                row![
                    icon(glyph, 12.0, scale, with_alpha(palette.surface_text, fade)),
                    label(words, 11.0, scale, with_alpha(palette.surface_text, fade)),
                ]
                .spacing(5.0 * scale)
                .align_y(Alignment::Center),
            )
            .padding([5.0 * scale, 10.0 * scale])
            .on_press(Message::Settings(SettingsMsg::HomePause))
            .style(move |_theme, status| pill_style(palette, fade, status, false)),
        );
    }
    let caption = match nav.backdrop.held {
        Some(reason) => format!("{} · paused by {reason}", nav.backdrop.caption),
        None => nav.backdrop.caption.clone(),
    };
    footer = footer.push(label(caption, 10.5, scale, with_alpha(palette.surface_text, 0.7 * fade)));
    footer = footer.push(container(text("")).width(Length::Fill));
    footer = footer.push(
        button(
            row![
                icon(
                    if nav.edit_cards { nav::ICON_CHECK } else { nav::ICON_EDIT },
                    12.0,
                    scale,
                    with_alpha(
                        if nav.edit_cards { palette.primary_text } else { palette.surface_text },
                        fade
                    )
                ),
                label(
                    if nav.edit_cards { "Done" } else { "Edit cards" },
                    11.0,
                    scale,
                    with_alpha(
                        if nav.edit_cards { palette.primary_text } else { palette.surface_text },
                        fade
                    )
                ),
            ]
            .spacing(5.0 * scale)
            .align_y(Alignment::Center),
        )
        .padding([5.0 * scale, 12.0 * scale])
        .on_press(Message::Settings(SettingsMsg::EditCards(!nav.edit_cards)))
        .style(move |_theme, status| pill_style(palette, fade, status, editing)),
    );
    let content = column![heading, grid, recent, footer.width(Length::Fixed(grid_width))]
        .spacing(18.0 * scale)
        .align_x(Alignment::Center);
    // Calm, roughly centred block (a little above the middle).
    let slots = count + usize::from(nav.edit_cards);
    let card_rows = slots.div_ceil(columns) as f32;
    let card_block = card_height + if nav.edit_cards { 26.0 * scale } else { 0.0 };
    let estimate =
        70.0 * scale + card_rows * (card_block + gap) + 140.0 * scale + 40.0 * scale + 54.0 * scale;
    let top = ((height - estimate) * 0.42).max(26.0 * scale);
    iced::widget::scrollable(
        container(content).width(Length::Fill).center_x(Length::Fill).padding(Padding {
            top,
            bottom: 30.0 * scale,
            ..Padding::ZERO
        }),
    )
    .direction(iced::widget::scrollable::Direction::Vertical(
        iced::widget::scrollable::Scrollbar::hidden(),
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn recent_card<'a>(
    nav: &NavView<'a>,
    focused: bool,
    width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    let mut list = column![].spacing(1.0 * scale);
    if nav.recent.is_empty() {
        list = list.push(label(
            "Nothing yet. Settings you change show up here, with a jump back to them.",
            11.5,
            scale,
            with_alpha(palette.surface_text, 0.62 * fade),
        ));
    }
    for (index, item) in nav.recent.iter().take(4).enumerate() {
        let first = index == 0 && focused;
        list = list.push(
            button(
                row![
                    label(item.title.clone(), 12.0, scale, with_alpha(palette.surface_text, fade))
                        .width(Length::Fill),
                    label(item.value.clone(), 11.0, scale, with_alpha(palette.primary, fade)),
                    icon(
                        nav::ICON_CHEVRON,
                        11.0,
                        scale,
                        with_alpha(palette.surface_text, 0.5 * fade)
                    ),
                ]
                .spacing(8.0 * scale)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([5.0 * scale, 8.0 * scale])
            .on_press(Message::Settings(SettingsMsg::OpenPageRow(item.page, item.row)))
            .style(move |_theme, status| {
                let hovered = matches!(status, button::Status::Hovered);
                crate::frontend::ui::rounded_button_style(
                    with_alpha(
                        palette.surface_text,
                        if hovered || first { 0.09 * fade } else { 0.0 },
                    ),
                    8.0,
                    (0.0, Color::TRANSPARENT),
                    Color::TRANSPARENT,
                    None,
                )
            }),
        );
    }
    container(
        column![
            row![
                icon(nav::ICON_HISTORY, 15.0, scale, with_alpha(palette.primary, fade)),
                label("Recently changed", 13.5, scale, with_alpha(palette.surface_text, fade)),
            ]
            .spacing(7.0 * scale)
            .align_y(Alignment::Center),
            list,
        ]
        .spacing(8.0 * scale),
    )
    .width(Length::Fixed(width))
    .padding(Padding {
        top: 12.0 * scale,
        right: 12.0 * scale,
        bottom: 10.0 * scale,
        left: 14.0 * scale,
    })
    .style(move |_| {
        crate::frontend::ui::surface_style(
            with_alpha(palette.surface, 0.66 * fade),
            14.0,
            (
                if focused { 1.6 } else { 1.0 },
                with_alpha(
                    if focused { palette.primary } else { palette.outline },
                    if focused { fade } else { 0.42 * fade },
                ),
            ),
            None,
        )
    })
    .into()
}
