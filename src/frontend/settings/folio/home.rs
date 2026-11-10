//! R1 / R1b (custom build): Home. The wallpaper (or a video) fills the
//! window under a dark veil. A big title and the latest changes sit at the
//! top, the middle stays empty so the picture can breathe, and the pinned
//! pages wait at the bottom as a row of numbered cards.

use iced::widget::{button, column, container, mouse_area, text};
use iced::{Alignment, Color, Element, Length, Padding};

use crate::app::Message;
use crate::frontend::components::with_alpha;
use crate::frontend::theme::Palette;
use crate::frontend::ui::text_button_style as quiet;
use crate::frontend::ui::{
    DISPLAY_FONT, MONO_FONT, NERD_FONT, TEXT_FONT, TEXT_STRONG_FONT, legible_type_scale, row, typed,
};

use super::super::nav::{self, GROUPS, PAGES};
use super::super::{SettingsFocus, SettingsMsg};
use super::shell::NavView;

/// Cards in one row on Home (the keyboard's Up/Down jump by this many).
pub(crate) const HOME_COLUMNS: usize = 6;

fn icon<'a>(glyph: &'static str, size: f32, scale: f32, color: Color) -> iced::widget::Text<'a> {
    text(glyph).font(NERD_FONT).size(size * legible_type_scale(scale)).color(color)
}

/// A quiet word that is a button.
fn word_button<'a>(
    glyph: Option<&'static str>,
    words: &'static str,
    message: SettingsMsg,
    strong: bool,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    let mut content = row![].spacing(6.0 * scale).align_y(Alignment::Center);
    if let Some(glyph) = glyph {
        content = content.push(text(glyph).font(NERD_FONT).size(11.0 * legible_type_scale(scale)));
    }
    content = content.push(text(words).font(MONO_FONT).size(10.5 * legible_type_scale(scale)));
    button(content)
        .padding([4.0 * scale, 7.0 * scale])
        .on_press(Message::Settings(message))
        .style(move |_theme, status| quiet(palette, fade, strong, false, status))
        .into()
}

/// One pinned page: number, icon, name, one sentence, and its group as a
/// tag. Lit (pointer or keyboard), "Open" takes the tag's place.
#[allow(clippy::too_many_arguments)]
fn shortcut<'a>(
    nav: &NavView<'a>,
    position: usize,
    page_index: usize,
    focused: bool,
    width: f32,
    height: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    let page = &PAGES[page_index];
    let hovered = nav.card_hover == Some(position);
    let lit = hovered || focused;
    let glow = if hovered {
        crate::frontend::animation::smoothstep(nav.card_fade.clamp(0.0, 1.0))
    } else {
        0.0
    };
    let header = row![
        typed(
            format!("{:02}", position + 1),
            MONO_FONT,
            10.0,
            scale,
            with_alpha(palette.surface_text, 0.4 * fade)
        ),
        iced::widget::Space::new().width(Length::Fill),
        icon(page.icon, 14.0, scale, with_alpha(palette.primary, fade)),
    ]
    .align_y(Alignment::Center);
    let name =
        typed(page.label, TEXT_STRONG_FONT, 13.5, scale, with_alpha(palette.surface_text, fade));
    let sentence =
        typed(page.sentence, TEXT_FONT, 11.0, scale, with_alpha(palette.surface_text, 0.62 * fade))
            .line_height(iced::widget::text::LineHeight::Relative(1.35))
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph);
    let footer: Element<'a, Message> = if nav.edit_cards {
        row![
            word_button(
                Some(nav::ICON_BACK),
                "",
                SettingsMsg::CardMove(position, true),
                false,
                scale,
                palette,
                fade
            ),
            word_button(
                Some(nav::ICON_CHEVRON),
                "",
                SettingsMsg::CardMove(position, false),
                false,
                scale,
                palette,
                fade
            ),
            iced::widget::Space::new().width(Length::Fill),
            word_button(
                None,
                "Remove",
                SettingsMsg::CardRemove(position),
                false,
                scale,
                palette,
                fade
            ),
        ]
        .align_y(Alignment::Center)
        .into()
    } else if lit {
        container(
            row![
                typed("Open", MONO_FONT, 9.5, scale, with_alpha(palette.primary_text, fade)),
                iced::widget::Space::new().width(Length::Fill),
                icon(nav::ICON_CHEVRON, 10.0, scale, with_alpha(palette.primary_text, fade)),
            ]
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([3.0 * scale, 7.0 * scale])
        .style(move |_| crate::frontend::ui::ink_fill(palette.primary, fade))
        .into()
    } else {
        row![
            container(typed(
                GROUPS[page.group].label,
                MONO_FONT,
                9.5,
                scale,
                with_alpha(palette.surface_text, 0.56 * fade)
            ))
            .padding([3.0 * scale, 7.0 * scale])
            .style(move |_| crate::frontend::ui::tag_style(palette, fade, false))
        ]
        .into()
    };
    let body = column![
        header,
        name,
        container(sentence).height(Length::Fill),
        container(footer).height(Length::Fixed(22.0 * scale)).align_y(Alignment::Center),
    ]
    .spacing(6.0 * scale);
    let card = container(body)
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .padding(11.0 * scale)
        .style(move |_| {
            let mut style = crate::frontend::ui::tile_style(palette, fade, glow, focused, false);
            // A light veil of its own, so the text holds on bright pictures.
            style.background = Some(iced::Background::Color(with_alpha(
                crate::frontend::ui::INK,
                (0.5 + 0.22 * glow) * fade,
            )));
            style
        });
    let mut area = mouse_area(card)
        .on_enter(Message::Settings(SettingsMsg::CardHover(Some(position))))
        .on_exit(Message::Settings(SettingsMsg::CardHover(None)));
    if !nav.edit_cards {
        area = area
            .on_press(Message::Settings(SettingsMsg::OpenPage(page_index)))
            .interaction(iced::mouse::Interaction::Pointer);
    }
    area.into()
}

/// The latest changes as plain lines with thin separators.
fn recent<'a>(
    nav: &NavView<'a>,
    focused: bool,
    width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    let mut list = column![
        typed(
            "Recently changed",
            MONO_FONT,
            10.5,
            scale,
            with_alpha(palette.surface_text, 0.5 * fade)
        ),
        super::shell::rule(palette, 0.3 * fade),
    ]
    .spacing(7.0 * scale);
    if nav.recent.is_empty() {
        list = list.push(
            typed(
                "Nothing yet. Settings you change show up here, with a way back to them.",
                TEXT_FONT,
                11.5,
                scale,
                with_alpha(palette.surface_text, 0.6 * fade),
            )
            .line_height(iced::widget::text::LineHeight::Relative(1.4)),
        );
    }
    for (index, item) in nav.recent.iter().take(4).enumerate() {
        let first = index == 0 && focused;
        list = list.push(
            button(
                row![
                    text(item.title.clone())
                        .font(TEXT_FONT)
                        .size(12.5 * legible_type_scale(scale))
                        .width(Length::Fill)
                        .wrapping(iced::widget::text::Wrapping::None),
                    typed(
                        item.value.clone(),
                        MONO_FONT,
                        11.0,
                        scale,
                        with_alpha(palette.primary, fade)
                    ),
                ]
                .spacing(10.0 * scale)
                .align_y(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([2.0 * scale, 0.0])
            .on_press(Message::Settings(SettingsMsg::OpenPageRow(item.page, item.row)))
            .style(move |_theme, status| quiet(palette, fade, first, false, status)),
        );
    }
    container(list).width(Length::Fixed(width)).into()
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
    let home_focus = focus == SettingsFocus::Home;
    let count = nav.home_cards.len();
    let slots = count + usize::from(nav.edit_cards);
    let columns = HOME_COLUMNS.min(slots.max(3));
    let gap = 12.0 * scale;
    let card_width = ((width - gap * (columns as f32 - 1.0)) / columns as f32).floor();
    let card_height = 128.0 * scale;
    let mut grid = column![].spacing(gap);
    let mut line = row![].spacing(gap);
    let mut in_line = 0;
    for (position, &page_index) in nav.home_cards.iter().enumerate() {
        line = line.push(shortcut(
            nav,
            position,
            page_index,
            home_focus && nav.card_focus == position,
            card_width,
            card_height,
            scale,
            palette,
            fade,
        ));
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
                icon(nav::ICON_PLUS, 16.0, scale, with_alpha(palette.primary, fade)),
                typed(
                    "Add a card",
                    TEXT_STRONG_FONT,
                    13.0,
                    scale,
                    with_alpha(palette.surface_text, fade)
                ),
                typed(
                    "Pick any page from the menus above.",
                    TEXT_FONT,
                    11.0,
                    scale,
                    with_alpha(palette.surface_text, 0.6 * fade)
                )
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
            ]
            .spacing(6.0 * scale),
        )
        .width(Length::Fixed(card_width))
        .height(Length::Fixed(card_height))
        .padding(11.0 * scale)
        .on_press(Message::Settings(SettingsMsg::MenuClick(1)))
        .style(move |_theme, status| {
            crate::frontend::ui::tile_button_style(palette, fade, focused, true, true, status)
        });
        line = line.push(add);
        in_line += 1;
    }
    if in_line > 0 {
        grid = grid.push(line);
    }
    let heading = column![
        text(if nav.edit_cards { "Edit your cards" } else { "Settings" })
            .font(DISPLAY_FONT)
            .size(34.0 * scale.clamp(0.85, 1.15))
            .color(with_alpha(palette.surface_text, fade)),
        typed(
            if nav.edit_cards {
                "The arrows on a card move it, Remove takes it off. With the keyboard: arrows choose, Shift+arrows move, Delete removes, Esc ends."
            } else {
                "Your pinned pages wait below. The menus above hold everything else, and Ctrl+K finds any setting by name."
            },
            TEXT_FONT,
            12.5,
            scale,
            with_alpha(palette.surface_text, 0.7 * fade),
        )
        .line_height(iced::widget::text::LineHeight::Relative(1.5))
        .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
    ]
    .spacing(10.0 * scale)
    .width(Length::Fixed((width * 0.4).clamp(280.0 * scale, 440.0 * scale)));
    let recent_width = (width * 0.3).clamp(240.0 * scale, 340.0 * scale);
    let top = row![
        heading,
        iced::widget::Space::new().width(Length::Fill),
        recent(
            nav,
            home_focus && !nav.edit_cards && nav.card_focus >= count,
            recent_width,
            scale,
            palette,
            fade
        ),
    ]
    .align_y(Alignment::Start);
    let mut footer = row![].spacing(6.0 * scale).align_y(Alignment::Center);
    if nav.backdrop.video {
        let (glyph, words) = if nav.backdrop.paused || nav.backdrop.held.is_some() {
            (nav::ICON_PLAY, "Play background")
        } else {
            (nav::ICON_PAUSE_SMALL, "Pause background")
        };
        footer = footer.push(word_button(
            Some(glyph),
            words,
            SettingsMsg::HomePause,
            false,
            scale,
            palette,
            fade,
        ));
    }
    let caption = match nav.backdrop.held {
        Some(reason) => format!("{} (paused by {reason})", nav.backdrop.caption),
        None => nav.backdrop.caption.clone(),
    };
    footer = footer.push(typed(
        caption,
        MONO_FONT,
        10.0,
        scale,
        with_alpha(palette.surface_text, 0.42 * fade),
    ));
    footer = footer.push(iced::widget::Space::new().width(Length::Fill));
    footer = footer.push(word_button(
        Some(if nav.edit_cards { nav::ICON_CHECK } else { nav::ICON_EDIT }),
        if nav.edit_cards { "Done" } else { "Edit cards" },
        SettingsMsg::EditCards(!nav.edit_cards),
        nav.edit_cards,
        scale,
        palette,
        fade,
    ));
    let card_rows = slots.div_ceil(columns).max(1) as f32;
    let needed = 150.0 * scale + card_rows * (card_height + gap) + 60.0 * scale;
    let pad = Padding {
        top: 34.0 * scale,
        bottom: 14.0 * scale,
        left: 24.0 * scale,
        right: 24.0 * scale,
    };
    if needed > height {
        // Too many cards for the window: everything scrolls.
        let content = column![top, grid, footer].spacing(22.0 * scale).width(Length::Fixed(width));
        return iced::widget::scrollable(
            container(content).width(Length::Fill).center_x(Length::Fill).padding(pad),
        )
        .direction(iced::widget::scrollable::Direction::Vertical(
            iced::widget::scrollable::Scrollbar::hidden(),
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    }
    let content = column![
        top,
        iced::widget::Space::new().height(Length::Fill),
        grid,
        container(footer).padding(Padding { top: 10.0 * scale, ..Padding::ZERO }),
    ]
    .width(Length::Fixed(width))
    .height(Length::Fill);
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .padding(pad)
        .into()
}
