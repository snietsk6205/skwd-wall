//! R1 (custom build): the Settings window: a small centred text-only top
//! bar with six groups, their menus (list or grid), a breadcrumb with Back,
//! one page under the bar, the command palette and the undo toast.

use iced::widget::{
    button, column, container, image, mouse_area, scrollable, stack, text, text_input,
};
use iced::{Alignment, Background, Color, Element, Length, Padding};

use crate::app::Message;
use crate::frontend::components::with_alpha;
use crate::frontend::theme::Palette;
use crate::frontend::ui::ghost_button_style as ghost;
use crate::frontend::ui::{AnchorSide, TYPE_SMALL, anchored, label, legible_type_scale, row};

use super::super::nav::{
    self, GROUPS, ICON_BACK, ICON_CHEVRON, ICON_GRID, ICON_LIST, ICON_SPARK, PAGES, PaletteEntry,
    PaletteValue,
};
use super::super::{SettingsFocus, SettingsMsg};

/// One "Recently changed" line.
#[derive(Clone)]
pub struct RecentItem {
    pub title: String,
    pub value: String,
    pub page: usize,
    pub row: usize,
}

/// The Home background (current wallpaper, a chosen picture or video).
#[derive(Clone, Default)]
pub struct Backdrop {
    pub picture: Option<iced::advanced::image::Handle>,
    pub blurred: Option<iced::advanced::image::Handle>,
    pub video: bool,
    pub paused: bool,
    /// Why a video does not play right now ("Low-power mode", ...).
    pub held: Option<&'static str>,
    pub caption: String,
}

pub struct PaletteView<'a> {
    pub query: &'a str,
    pub selected: usize,
    pub results: Vec<&'a PaletteEntry>,
    pub empty_query: bool,
}

/// Everything the R1 window needs from the app.
pub struct NavView<'a> {
    pub page: Option<usize>,
    pub can_go_back: bool,
    pub menu: Option<usize>,
    pub menu_item: usize,
    pub menu_fade: f32,
    pub menu_unavailable: &'a [usize],
    pub menu_grid: bool,
    pub bar_item: usize,
    pub page_fade: f32,
    pub simple: bool,
    pub hides_advanced: bool,
    pub edit_cards: bool,
    pub card_focus: usize,
    pub card_hover: Option<usize>,
    pub card_fade: f32,
    pub home_cards: Vec<usize>,
    pub recent: Vec<RecentItem>,
    pub palette: Option<PaletteView<'a>>,
    pub toast: Option<(String, bool)>,
    pub toast_fade: f32,
    pub bubble: Option<super::rows::BubbleView>,
    pub backdrop: Backdrop,
    pub bar_blur: bool,
    pub more: &'a std::collections::HashSet<String>,
    pub playlist: String,
    /// Low-power mode: no animations.
    pub still: bool,
}

pub(super) fn bar_height(scale: f32) -> f32 {
    54.0 * scale
}

fn icon<'a>(glyph: &'static str, size: f32, scale: f32, color: Color) -> iced::widget::Text<'a> {
    text(glyph)
        .font(crate::frontend::ui::NERD_FONT)
        .size(size * legible_type_scale(scale))
        .color(color)
}

/// The top bar: groups in the centre, view switch, palette and close on the
/// right.
pub(super) fn top_bar<'a>(
    nav: &NavView<'a>,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    bar_focused: bool,
) -> Element<'a, Message> {
    let current_group = nav.page.map_or(0, |page| PAGES[page].group);
    let mut groups = row![].spacing(4.0 * scale).align_y(Alignment::Center);
    for (index, group) in GROUPS.iter().enumerate() {
        let active = index == current_group;
        let focused = bar_focused && nav.bar_item == index;
        let open = nav.menu == Some(index);
        let mut title = row![
            text(group.label)
                .font(crate::frontend::ui::UI_FONT)
                .size(13.0 * legible_type_scale(scale))
        ]
        .spacing(5.0 * scale)
        .align_y(Alignment::Center);
        if index > 0 {
            title = title.push(
                text(nav::ICON_CHEVRON_DOWN)
                    .font(crate::frontend::ui::NERD_FONT)
                    .size(10.0 * legible_type_scale(scale)),
            );
        }
        let underline =
            container(text("")).width(Length::Fill).height(Length::Fixed(2.0)).style(move |_| {
                crate::frontend::ui::bg_style(Background::Color(with_alpha(
                    palette.primary,
                    if active { fade } else { 0.0 },
                )))
            });
        let message =
            if index == 0 { SettingsMsg::OpenHome } else { SettingsMsg::MenuClick(index) };
        let entry =
            button(column![title, underline].spacing(4.0 * scale).align_x(Alignment::Center))
                .padding([6.0 * scale, 10.0 * scale])
                .on_press(Message::Settings(message))
                .style(move |_theme, status| ghost(palette, fade, active || open, focused, status));
        let entry: Element<'a, Message> = if index == 0 {
            entry.into()
        } else {
            mouse_area(entry)
                .on_enter(Message::Settings(SettingsMsg::MenuHover(index)))
                .on_exit(Message::Settings(SettingsMsg::MenuLeave))
                .into()
        };
        let panel = (nav.menu == Some(index)).then(|| menu_panel(nav, index, scale, palette));
        groups = groups.push(anchored(entry, panel, AnchorSide::Below, 6.0 * scale));
    }
    let view_switch = {
        let focused = bar_focused && nav.bar_item == 6;
        let chip = |text_: &'static str, on: bool| {
            container(label(
                text_,
                11.0,
                scale,
                with_alpha(
                    if on { palette.primary_text } else { palette.surface_text },
                    if on { fade } else { 0.7 * fade },
                ),
            ))
            .padding([4.0 * scale, 9.0 * scale])
            .style(move |_| {
                crate::frontend::ui::surface_style(
                    with_alpha(palette.primary, if on { 0.92 * fade } else { 0.0 }),
                    999.0,
                    (0.0, Color::TRANSPARENT),
                    None,
                )
            })
        };
        button(row![chip("Simple", nav.simple), chip("Advanced", !nav.simple)].spacing(2.0 * scale))
            .padding(2.0 * scale)
            .on_press(Message::Settings(SettingsMsg::ToggleView))
            .style(move |_theme, status| {
                let mut style = ghost(palette, fade, false, focused, status);
                style.border = iced::Border {
                    radius: (999.0).into(),
                    width: 1.0,
                    color: with_alpha(
                        if focused { palette.primary } else { palette.outline },
                        0.7 * fade,
                    ),
                };
                style
            })
    };
    let search = {
        let focused = bar_focused && nav.bar_item == 7;
        button(
            row![
                icon(nav::ICON_SEARCH, 13.0, scale, with_alpha(palette.surface_text, 0.8 * fade)),
                label("Find", 12.0, scale, with_alpha(palette.surface_text, 0.8 * fade)),
                container(label(
                    "Ctrl K",
                    9.5,
                    scale,
                    with_alpha(palette.surface_text, 0.55 * fade)
                ))
                .padding([2.0 * scale, 5.0 * scale])
                .style(move |_| crate::frontend::ui::surface_style(
                    Color::TRANSPARENT,
                    5.0,
                    (1.0, with_alpha(palette.outline, 0.6 * fade)),
                    None
                )),
            ]
            .spacing(7.0 * scale)
            .align_y(Alignment::Center),
        )
        .padding([6.0 * scale, 10.0 * scale])
        .on_press(Message::Settings(SettingsMsg::PaletteOpen))
        .style(move |_theme, status| ghost(palette, fade, false, focused, status))
    };
    let close =
        button(icon(nav::ICON_CLOSE, 15.0, scale, with_alpha(palette.surface_text, 0.8 * fade)))
            .padding([6.0 * scale, 8.0 * scale])
            .on_press(Message::ToggleSettings)
            .style(move |_theme, status| ghost(palette, fade, false, false, status));
    let hide = button(icon(
        crate::frontend::ui::EYE_CLOSED,
        14.0,
        scale,
        with_alpha(palette.surface_text, 0.7 * fade),
    ))
    .padding([6.0 * scale, 8.0 * scale])
    .on_press(Message::Stash)
    .style(move |_theme, status| ghost(palette, fade, false, false, status));
    let brand = row![
        icon(ICON_SPARK, 14.0, scale, with_alpha(palette.primary, fade)),
        label("Settings", 13.0, scale, with_alpha(palette.surface_text, 0.9 * fade)),
    ]
    .spacing(7.0 * scale)
    .align_y(Alignment::Center);
    let left = container(brand)
        .width(Length::FillPortion(2))
        .padding(Padding { left: 18.0 * scale, ..Padding::ZERO });
    let right = container(
        row![view_switch, search, hide, close].spacing(6.0 * scale).align_y(Alignment::Center),
    )
    .width(Length::FillPortion(2))
    .align_x(crate::frontend::ui::end())
    .padding(Padding { right: 12.0 * scale, ..Padding::ZERO });
    let content = row![left, container(groups).width(Length::Shrink), right]
        .align_y(Alignment::Center)
        .height(Length::Fixed(bar_height(scale)));
    let bar_bg = if nav.bar_blur { 0.35 } else { 0.0 };
    container(content)
        .width(Length::Fill)
        .style(move |_| {
            crate::frontend::ui::surface_style(
                with_alpha(palette.surface, bar_bg * fade),
                0.0,
                (0.0, Color::TRANSPARENT),
                None,
            )
        })
        .into()
}

/// A group's menu: icon, name and one sentence per page.
fn menu_panel<'a>(
    nav: &NavView<'a>,
    group: usize,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let fade = nav.menu_fade.clamp(0.0, 1.0);
    let ease = crate::frontend::animation::smoothstep(fade);
    let pages: Vec<usize> = nav::group_pages(group).map(|(index, _)| index).collect();
    let grid = nav.menu_grid;
    let columns = if grid { 3 } else { 1 };
    let item_width = if grid { 212.0 } else { 372.0 } * scale;
    let mut lines = column![].spacing(2.0 * scale);
    let mut current = row![].spacing(4.0 * scale);
    let mut in_row = 0;
    for (position, &page_index) in pages.iter().enumerate() {
        let page = &PAGES[page_index];
        let selected = nav.menu_item == position;
        let here = nav.page == Some(page_index);
        let unavailable = nav.menu_unavailable.contains(&page_index);
        let advanced = nav.simple && nav::ADVANCED_PAGES.contains(&page.key);
        let note = if unavailable { "Not used with your current settings." } else { page.sentence };
        let name_row = row![label(
            page.label,
            13.0,
            scale,
            with_alpha(palette.surface_text, if unavailable { 0.5 } else { 1.0 } * ease)
        ),]
        .push_maybe(advanced.then(|| {
            container(label("Advanced", 8.5, scale, with_alpha(palette.surface_text, 0.6 * ease)))
                .padding([1.0 * scale, 5.0 * scale])
                .style(move |_| {
                    crate::frontend::ui::surface_style(
                        Color::TRANSPARENT,
                        4.0,
                        (1.0, with_alpha(palette.outline, 0.6 * ease)),
                        None,
                    )
                })
        }))
        .spacing(6.0 * scale)
        .align_y(Alignment::Center);
        let copy = column![
            name_row,
            label(note, TYPE_SMALL, scale, with_alpha(palette.surface_text, 0.58 * ease))
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
        ]
        .spacing(2.0 * scale);
        let glyph = container(icon(page.icon, 17.0, scale, with_alpha(palette.primary, ease)))
            .width(Length::Fixed(30.0 * scale))
            .height(Length::Fixed(30.0 * scale))
            .center(Length::Fixed(30.0 * scale))
            .style(move |_| {
                crate::frontend::ui::surface_style(
                    with_alpha(palette.primary, 0.12 * ease),
                    8.0,
                    (0.0, Color::TRANSPARENT),
                    None,
                )
            });
        let body: Element<'a, Message> = if grid {
            column![glyph, copy].spacing(8.0 * scale).into()
        } else {
            row![glyph, copy].spacing(11.0 * scale).align_y(Alignment::Center).into()
        };
        let message = if nav.edit_cards && nav.page.is_none() {
            SettingsMsg::CardAdd(page_index)
        } else {
            SettingsMsg::OpenPage(page_index)
        };
        let item = button(body)
            .width(Length::Fixed(item_width))
            .padding([8.0 * scale, 10.0 * scale])
            .on_press(Message::Settings(message))
            .style(move |_theme, status| {
                let mut style = ghost(palette, ease, here, selected, status);
                if here && !selected {
                    style.background =
                        Some(Background::Color(with_alpha(palette.primary, 0.07 * ease)));
                }
                style
            });
        current = current.push(item);
        in_row += 1;
        if in_row == columns || position + 1 == pages.len() {
            lines = lines.push(std::mem::replace(&mut current, row![].spacing(4.0 * scale)));
            in_row = 0;
        }
    }
    let group_info = &GROUPS[group];
    let header = row![
        label(group_info.label, 11.0, scale, with_alpha(palette.primary, ease)),
        container(text("")).width(Length::Fill),
        label(
            if grid { "grid" } else { "list" },
            9.5,
            scale,
            with_alpha(palette.surface_text, 0.45 * ease)
        ),
        icon(
            if grid { ICON_GRID } else { ICON_LIST },
            10.0,
            scale,
            with_alpha(palette.surface_text, 0.45 * ease)
        ),
    ]
    .spacing(5.0 * scale)
    .align_y(Alignment::Center);
    let footer = label(
        if nav.edit_cards && nav.page.is_none() {
            "Click a page to pin it on Home."
        } else {
            "↑↓ ←→ choose · Enter open · Esc close"
        },
        9.5,
        scale,
        with_alpha(palette.surface_text, 0.42 * ease),
    );
    let panel_width =
        item_width * columns as f32 + 4.0 * scale * (columns as f32 - 1.0) + 20.0 * scale;
    let panel = container(column![header, lines, footer].spacing(8.0 * scale))
        .width(Length::Fixed(panel_width))
        .padding(Padding {
            top: 12.0 * scale,
            right: 10.0 * scale,
            bottom: 10.0 * scale,
            left: 10.0 * scale,
        })
        .style(move |_| {
            crate::frontend::ui::surface_style(
                with_alpha(palette.surface_container, ease),
                14.0,
                (1.0, with_alpha(palette.outline, 0.55 * ease)),
                Some((with_alpha(Color::BLACK, 0.4 * ease), 14.0, 36.0)),
            )
        });
    let slide = 10.0 * (1.0 - ease) * scale;
    mouse_area(container(panel).padding(Padding { top: slide, ..Padding::ZERO }))
        .on_enter(Message::Settings(SettingsMsg::MenuHover(group)))
        .on_exit(Message::Settings(SettingsMsg::MenuLeave))
        .into()
}

/// "← Back   Motion › Transitions › Combine two".
pub(super) fn breadcrumb<'a>(
    nav: &NavView<'a>,
    page: usize,
    segment: Option<String>,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    let info = &PAGES[page];
    let link = |content: Element<'a, Message>, message: SettingsMsg| {
        button(content)
            .padding([3.0 * scale, 6.0 * scale])
            .on_press(Message::Settings(message))
            .style(move |_theme, status| ghost(palette, fade, false, false, status))
    };
    let sep = || icon(ICON_CHEVRON, 11.0, scale, with_alpha(palette.surface_text, 0.4 * fade));
    let back = link(
        row![
            icon(ICON_BACK, 13.0, scale, with_alpha(palette.surface_text, 0.85 * fade)),
            label(
                if nav.can_go_back { "Back" } else { "Home" },
                11.5,
                scale,
                with_alpha(palette.surface_text, 0.85 * fade)
            ),
        ]
        .spacing(5.0 * scale)
        .align_y(Alignment::Center)
        .into(),
        SettingsMsg::Back,
    );
    let mut trail = row![
        back,
        container(text("")).width(Length::Fixed(10.0 * scale)),
        link(
            label(
                GROUPS[info.group].label,
                11.5,
                scale,
                with_alpha(palette.surface_text, 0.62 * fade)
            )
            .into(),
            SettingsMsg::MenuClick(info.group)
        ),
        sep(),
        link(
            label(
                info.label,
                11.5,
                scale,
                with_alpha(
                    palette.surface_text,
                    if segment.is_some() { 0.62 } else { 0.95 } * fade
                )
            )
            .into(),
            SettingsMsg::OpenPage(page)
        ),
    ]
    .spacing(2.0 * scale)
    .align_y(Alignment::Center);
    if let Some(segment) = segment {
        trail = trail.push(sep());
        trail = trail.push(
            container(label(segment, 11.5, scale, with_alpha(palette.surface_text, 0.95 * fade)))
                .padding([3.0 * scale, 6.0 * scale]),
        );
    }
    trail.into()
}

/// The page under the bar: breadcrumb, title, sentence, then the rows.
#[allow(clippy::too_many_arguments)]
pub(super) fn page_frame<'a>(
    nav: &NavView<'a>,
    page: usize,
    segment: Option<String>,
    hidden: usize,
    content: Element<'a, Message>,
    width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    let info = &PAGES[page];
    let slide = if nav.still {
        0.0
    } else {
        14.0 * (1.0 - crate::frontend::animation::smoothstep(nav.page_fade)) * scale
    };
    let page_fade =
        fade * if nav.still { 1.0 } else { 0.25 + 0.75 * nav.page_fade.clamp(0.0, 1.0) };
    let mut header = column![
        breadcrumb(nav, page, segment, scale, palette, page_fade),
        row![
            container(icon(info.icon, 22.0, scale, with_alpha(palette.primary, page_fade)))
                .padding(Padding { top: 4.0 * scale, ..Padding::ZERO }),
            column![
                text(info.label)
                    .font(crate::frontend::ui::UI_FONT)
                    .size(30.0 * scale.clamp(0.85, 1.1))
                    .color(with_alpha(palette.surface_text, page_fade)),
                label(
                    info.sentence,
                    12.0,
                    scale,
                    with_alpha(palette.surface_text, 0.62 * page_fade)
                ),
            ]
            .spacing(2.0 * scale),
        ]
        .spacing(12.0 * scale)
        .align_y(Alignment::Start),
    ]
    .spacing(14.0 * scale);
    if hidden > 0 {
        header = header.push(
            row![
                label(
                    format!(
                        "Simple view hides {hidden} advanced setting{} here.",
                        if hidden == 1 { "" } else { "s" }
                    ),
                    TYPE_SMALL,
                    scale,
                    with_alpha(palette.surface_text, 0.5 * page_fade),
                ),
                button(label(
                    "Show them",
                    TYPE_SMALL,
                    scale,
                    with_alpha(palette.primary, page_fade)
                ))
                .padding([2.0 * scale, 6.0 * scale])
                .on_press(Message::Settings(SettingsMsg::ShowAllRows))
                .style(move |_theme, status| ghost(palette, page_fade, false, false, status)),
            ]
            .spacing(6.0 * scale)
            .align_y(Alignment::Center),
        );
    }
    let body = column![
        header,
        crate::frontend::ui::folio_horizontal_rule(with_alpha(palette.outline, 0.3 * page_fade)),
        content
    ]
    .spacing(16.0 * scale)
    .width(Length::Fixed(width));
    let padded = container(body).width(Length::Fill).center_x(Length::Fill).padding(Padding {
        top: 18.0 * scale + slide,
        right: 24.0 * scale,
        bottom: 64.0 * scale,
        left: 24.0 * scale,
    });
    scrollable(padded)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new().width(4.0 * scale).scroller_width(4.0 * scale),
        ))
        .id(crate::frontend::ui::pane_id(super::folio::READING_PANE))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// The command palette, over everything.
pub(super) fn palette_layer<'a>(
    view: &PaletteView<'a>,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    let input = text_input("Find a setting, page, transition, layout or action…", view.query)
        .id(super::super::palette_input_id())
        .font(crate::frontend::ui::UI_FONT)
        .on_input(|value| Message::Settings(SettingsMsg::PaletteInput(value)))
        .on_submit(Message::Settings(SettingsMsg::PaletteSubmit))
        .size(16.0 * legible_type_scale(scale))
        .padding([10.0 * scale, 12.0 * scale])
        .width(Length::Fill)
        .style(move |_theme, _status| {
            crate::frontend::ui::workbench_input_style(
                with_alpha(palette.surface_text, fade),
                with_alpha(palette.primary, fade),
            )
        });
    let mut list = column![].spacing(2.0 * scale);
    if view.empty_query {
        list = list.push(label(
            "Recent and suggested",
            10.0,
            scale,
            with_alpha(palette.surface_text, 0.5 * fade),
        ));
    }
    if view.results.is_empty() {
        list = list.push(
            container(label(
                "Nothing found. Try fewer letters or another word.",
                12.0,
                scale,
                with_alpha(palette.surface_text, 0.6 * fade),
            ))
            .padding([14.0 * scale, 6.0 * scale]),
        );
    }
    for (position, entry) in view.results.iter().enumerate() {
        let selected = position == view.selected;
        let value: Element<'a, Message> = match &entry.value {
            PaletteValue::Toggle { value, .. } => {
                let on = *value;
                button(
                    container(label(
                        if on { "On" } else { "Off" },
                        11.0,
                        scale,
                        with_alpha(
                            if on { palette.primary_text } else { palette.surface_text },
                            fade,
                        ),
                    ))
                    .padding([3.0 * scale, 10.0 * scale])
                    .style(move |_| {
                        crate::frontend::ui::surface_style(
                            with_alpha(
                                if on { palette.primary } else { palette.surface_text },
                                if on { 0.95 * fade } else { 0.12 * fade },
                            ),
                            999.0,
                            (0.0, Color::TRANSPARENT),
                            None,
                        )
                    }),
                )
                .padding(0)
                .on_press(Message::Settings(SettingsMsg::PaletteChange(position, None)))
                .style(|_theme, _status| button::Style::default())
                .into()
            }
            PaletteValue::Choice { options, current, .. } => {
                let mut chips = row![].spacing(3.0 * scale);
                for (key, text_) in options.iter().take(5) {
                    let on = key == current;
                    chips = chips.push(
                        button(label(
                            text_.clone(),
                            10.5,
                            scale,
                            with_alpha(
                                if on { palette.primary_text } else { palette.surface_text },
                                fade,
                            ),
                        ))
                        .padding([3.0 * scale, 7.0 * scale])
                        .on_press(Message::Settings(SettingsMsg::PaletteChange(
                            position,
                            Some(key.clone()),
                        )))
                        .style(move |_theme, status| {
                            let hovered = matches!(status, button::Status::Hovered);
                            crate::frontend::ui::rounded_button_style(
                                with_alpha(
                                    if on { palette.primary } else { palette.surface_text },
                                    if on {
                                        0.95 * fade
                                    } else if hovered {
                                        0.16 * fade
                                    } else {
                                        0.07 * fade
                                    },
                                ),
                                999.0,
                                (0.0, Color::TRANSPARENT),
                                Color::TRANSPARENT,
                                None,
                            )
                        }),
                    );
                }
                if options.len() > 5 {
                    chips = chips.push(label(
                        format!("+{}", options.len() - 5),
                        10.0,
                        scale,
                        with_alpha(palette.surface_text, 0.5 * fade),
                    ));
                }
                chips.into()
            }
            PaletteValue::None => {
                label(entry.kind.label(), 9.5, scale, with_alpha(palette.surface_text, 0.45 * fade))
                    .into()
            }
        };
        let copy = column![
            label(entry.title.clone(), 13.0, scale, with_alpha(palette.surface_text, fade)),
            label(entry.crumb.clone(), 10.0, scale, with_alpha(palette.primary, 0.75 * fade)),
        ]
        .spacing(1.0 * scale)
        .width(Length::Fill);
        let item = button(
            row![
                container(icon(entry.icon, 15.0, scale, with_alpha(palette.primary, 0.9 * fade)))
                    .width(Length::Fixed(24.0 * scale)),
                copy,
                value,
            ]
            .spacing(10.0 * scale)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([7.0 * scale, 10.0 * scale])
        .on_press(Message::Settings(SettingsMsg::PaletteRun(position)))
        .style(move |_theme, status| ghost(palette, fade, false, selected, status));
        list = list.push(
            mouse_area(item).on_enter(Message::Settings(SettingsMsg::PaletteHover(position))),
        );
    }
    let hints = label(
        "↑↓ choose · Tab change in place · Enter go there · Esc close",
        9.5,
        scale,
        with_alpha(palette.surface_text, 0.45 * fade),
    );
    let card = container(column![input, list, hints].spacing(10.0 * scale))
        .width(Length::Fixed(640.0 * scale))
        .padding(12.0 * scale)
        .style(move |_| {
            crate::frontend::ui::surface_style(
                with_alpha(palette.surface_container, fade),
                16.0,
                (1.0, with_alpha(palette.outline, 0.6 * fade)),
                Some((with_alpha(Color::BLACK, 0.5 * fade), 20.0, 48.0)),
            )
        });
    let scrim =
        mouse_area(container(text("")).width(Length::Fill).height(Length::Fill).style(move |_| {
            crate::frontend::ui::bg_style(Background::Color(with_alpha(Color::BLACK, 0.35 * fade)))
        }))
        .on_press(Message::Settings(SettingsMsg::PaletteClose));
    stack![
        scrim,
        container(iced::widget::opaque(card))
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding(Padding { top: 70.0 * scale, ..Padding::ZERO }),
    ]
    .into()
}

/// "Changed X to Y — Undo" at the bottom of the window.
pub(super) fn toast_layer(
    text_: String,
    can_undo: bool,
    scale: f32,
    palette: &Palette,
    fade: f32,
) -> Element<'_, Message> {
    let ease = crate::frontend::animation::smoothstep(fade.clamp(0.0, 1.0));
    let mut content = row![
        icon(nav::ICON_HISTORY, 14.0, scale, with_alpha(palette.primary, ease)),
        label(text_, 12.0, scale, with_alpha(palette.surface_text, ease)),
    ]
    .spacing(9.0 * scale)
    .align_y(Alignment::Center);
    if can_undo {
        content = content.push(
            button(
                row![
                    icon(nav::ICON_UNDO, 13.0, scale, with_alpha(palette.primary_text, ease)),
                    label("Undo", 12.0, scale, with_alpha(palette.primary_text, ease)),
                    label("Ctrl Z", 9.0, scale, with_alpha(palette.primary_text, 0.7 * ease)),
                ]
                .spacing(5.0 * scale)
                .align_y(Alignment::Center),
            )
            .padding([5.0 * scale, 10.0 * scale])
            .on_press(Message::Settings(SettingsMsg::Undo))
            .style(move |_theme, status| {
                let hovered = matches!(status, button::Status::Hovered);
                crate::frontend::ui::rounded_button_style(
                    with_alpha(palette.primary, if hovered { ease } else { 0.88 * ease }),
                    999.0,
                    (0.0, Color::TRANSPARENT),
                    Color::TRANSPARENT,
                    None,
                )
            }),
        );
    }
    let pill = container(content)
        .padding(Padding {
            top: 7.0 * scale,
            right: 8.0 * scale,
            bottom: 7.0 * scale,
            left: 14.0 * scale,
        })
        .style(move |_| {
            crate::frontend::ui::surface_style(
                with_alpha(palette.surface_container, ease),
                999.0,
                (1.0, with_alpha(palette.outline, 0.6 * ease)),
                Some((with_alpha(Color::BLACK, 0.4 * ease), 8.0, 24.0)),
            )
        });
    container(iced::widget::opaque(pill))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::End)
        .padding(Padding { bottom: 18.0 * scale + 10.0 * (1.0 - ease) * scale, ..Padding::ZERO })
        .into()
}

/// The picture behind Home (sharp) or behind the bar (blurred).
pub(super) fn backdrop_image<'a>(
    handle: iced::advanced::image::Handle,
    opacity: f32,
) -> Element<'a, Message> {
    image(handle)
        .width(Length::Fill)
        .height(Length::Fill)
        .content_fit(iced::ContentFit::Cover)
        .opacity(opacity)
        .into()
}

/// Whether the top bar has keyboard focus.
pub(super) fn bar_has_focus(focus: SettingsFocus) -> bool {
    focus == SettingsFocus::Bar
}
