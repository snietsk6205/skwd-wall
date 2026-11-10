//! R1 (custom build): the Settings window: a small centred text-only top
//! bar with six groups, their menus (list or grid), a breadcrumb with Back,
//! one page under the bar, the command palette and the undo toast.

use iced::widget::{
    button, column, container, image, mouse_area, scrollable, stack, text, text_input,
};
use iced::{Alignment, Color, Element, Length, Padding};

use crate::app::Message;
use crate::frontend::components::with_alpha;
use crate::frontend::theme::Palette;
use crate::frontend::ui::text_button_style as quiet;
use crate::frontend::ui::{
    AnchorSide, DISPLAY_FONT, MONO_FONT, TEXT_FONT, TEXT_STRONG_FONT, anchored, legible_type_scale,
    row, typed,
};

use super::super::nav::{self, GROUPS, ICON_BACK, PAGES, PaletteEntry, PaletteValue};
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

/// R1b: the one live preview in the big overlay.
#[derive(Clone)]
pub struct BigPreview {
    pub title: String,
    pub desc: String,
    pub frame: Option<iced::advanced::image::Handle>,
    pub fade: f32,
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
    /// R1b: the card under the pointer (row) and how far its glow is.
    pub tile_hover: Option<(usize, usize)>,
    pub tile_fade: f32,
    /// "Preview style: follows the cursor", and that preview when a card
    /// has the pointer.
    pub follow_cursor: bool,
    pub cursor: Option<super::cards::CursorView<'a>>,
    /// Middle frames of the card strips, and the frame of the one that plays.
    pub pictures: &'a std::collections::HashMap<String, iced::advanced::image::Handle>,
    pub playing: Option<(&'a str, &'a iced::advanced::image::Handle)>,
    /// Strips the background renderer still has to draw.
    pub strips_pending: usize,
    /// A page of the open menu that has a picture (page index, picture).
    pub menu_picture: Option<(usize, iced::advanced::image::Handle)>,
    pub big: Option<BigPreview>,
    /// The faint grid behind pages.
    pub grid: bool,
    pub card_layout: super::cards::CardLayout,
    pub backdrop: Backdrop,
    pub bar_blur: bool,
    pub more: &'a std::collections::HashSet<String>,
    /// Low-power mode: no animations.
    pub still: bool,
}

pub(super) fn bar_height(scale: f32) -> f32 {
    46.0 * scale
}

fn icon<'a>(glyph: &'static str, size: f32, scale: f32, color: Color) -> iced::widget::Text<'a> {
    text(glyph)
        .font(crate::frontend::ui::NERD_FONT)
        .size(size * legible_type_scale(scale))
        .color(color)
}

/// Text that takes its colour from the button around it (so it brightens
/// on hover).
fn plain<'a>(
    content: impl iced::widget::text::IntoFragment<'a>,
    font: iced::Font,
    size: f32,
    scale: f32,
) -> iced::widget::Text<'a> {
    text(content).font(font).size(size * legible_type_scale(scale))
}

fn spacer<'a>() -> Element<'a, Message> {
    iced::widget::Space::new().width(Length::Fill).into()
}

/// A one-pixel horizontal line.
pub(super) fn rule(palette: &Palette, opacity: f32) -> Element<'_, Message> {
    container(text(""))
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(move |_| crate::frontend::ui::hairline_style(palette, opacity))
        .into()
}

/// The top bar: small text-only group names in the centre (the open one
/// bright with a thin underline); Find, the view switch, hide and close as
/// quiet text on the right.
pub(super) fn top_bar<'a>(
    nav: &NavView<'a>,
    width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    bar_focused: bool,
) -> Element<'a, Message> {
    let current_group = nav.page.map_or(0, |page| PAGES[page].group);
    let bar_item = nav.bar_item;
    // A narrow window: shorter names, no key hint, groups from the left.
    let compact = width < 1000.0 * scale;
    let mut groups = row![].spacing(2.0 * scale).align_y(Alignment::Center);
    for (index, group) in GROUPS.iter().enumerate() {
        let active = index == current_group;
        let focused = bar_focused && nav.bar_item == index;
        let open = nav.menu == Some(index);
        let underline =
            container(text("")).width(Length::Fill).height(Length::Fixed(1.0)).style(move |_| {
                crate::frontend::ui::ink_fill(palette.primary, if active { fade } else { 0.0 })
            });
        let message =
            if index == 0 { SettingsMsg::OpenHome } else { SettingsMsg::MenuClick(index) };
        let entry = button(
            column![
                plain(
                    if compact {
                        group.label.split(" & ").next().unwrap_or(group.label)
                    } else {
                        group.label
                    },
                    MONO_FONT,
                    12.0,
                    scale
                ),
                underline
            ]
            .spacing(5.0 * scale)
            .align_x(Alignment::Center),
        )
        .padding([7.0 * scale, if compact { 7.0 } else { 11.0 } * scale])
        .on_press(Message::Settings(message))
        .style(move |_theme, status| quiet(palette, fade, active || open, focused, status));
        let entry: Element<'a, Message> = if index == 0 {
            entry.into()
        } else {
            mouse_area(entry)
                .on_enter(Message::Settings(SettingsMsg::MenuHover(index)))
                .on_exit(Message::Settings(SettingsMsg::MenuLeave))
                .into()
        };
        let panel = (nav.menu == Some(index)).then(|| menu_panel(nav, index, scale, palette));
        groups = groups.push(anchored(entry, panel, AnchorSide::Below, 8.0 * scale));
    }
    let dim = with_alpha(palette.surface_text, 0.34 * fade);
    let search = button(
        row![
            plain("Find", MONO_FONT, 11.5, scale),
            typed(if compact { "" } else { "Ctrl K" }, MONO_FONT, 10.0, scale, dim)
        ]
        .spacing(if compact { 0.0 } else { 7.0 * scale })
        .align_y(Alignment::Center),
    )
    .padding([6.0 * scale, 9.0 * scale])
    .on_press(Message::Settings(SettingsMsg::PaletteOpen))
    .style(move |_theme, status| quiet(palette, fade, false, bar_focused && bar_item == 7, status));
    let word = |name: &'static str, on: bool| {
        typed(
            name,
            MONO_FONT,
            11.5,
            scale,
            with_alpha(palette.surface_text, if on { fade } else { 0.36 * fade }),
        )
    };
    let view_switch = button(if compact {
        // Only the view in use; a click switches to the other one.
        row![word(if nav.simple { "Simple" } else { "Advanced" }, true)]
    } else {
        row![
            word("Simple", nav.simple),
            typed("/", MONO_FONT, 11.5, scale, dim),
            word("Advanced", !nav.simple)
        ]
        .spacing(6.0 * scale)
        .align_y(Alignment::Center)
    })
    .padding([6.0 * scale, 9.0 * scale])
    .on_press(Message::Settings(SettingsMsg::ToggleView))
    .style(move |_theme, status| quiet(palette, fade, false, bar_focused && bar_item == 6, status));
    let glyph_button = |glyph: &'static str, size: f32, message: Message| {
        button(
            text(glyph).font(crate::frontend::ui::NERD_FONT).size(size * legible_type_scale(scale)),
        )
        .padding([6.0 * scale, 8.0 * scale])
        .on_press(message)
        .style(move |_theme, status| quiet(palette, fade, false, false, status))
    };
    let right = container(
        row![
            search,
            view_switch,
            glyph_button(crate::frontend::ui::EYE_CLOSED, 13.0, Message::Stash),
            glyph_button(nav::ICON_CLOSE, 14.0, Message::ToggleSettings),
        ]
        .spacing(4.0 * scale)
        .align_y(Alignment::Center),
    )
    .width(Length::FillPortion(1))
    .align_x(crate::frontend::ui::end())
    .padding(Padding { right: 10.0 * scale, ..Padding::ZERO });
    // Wide: the groups sit in the middle. Narrow: they start at the left.
    let left = container(text("")).width(if compact {
        Length::Fixed(8.0 * scale)
    } else {
        Length::FillPortion(1)
    });
    let content = row![left, container(groups).width(Length::Shrink), right]
        .align_y(Alignment::Center)
        .height(Length::Fixed(bar_height(scale)));
    container(content).width(Length::Fill).into()
}

/// A group's menu: a panel of numbered page cards with thin outlines; the
/// hovered or focused card lights up. One column (list) or three (grid).
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
    let gap = 6.0 * scale;
    let item_width = if grid { 196.0 } else { 408.0 } * scale;
    let mut lines = column![].spacing(gap);
    let mut current = row![].spacing(gap);
    let mut in_row = 0;
    for (position, &page_index) in pages.iter().enumerate() {
        let page = &PAGES[page_index];
        let selected = nav.menu_item == position;
        let here = nav.page == Some(page_index);
        let unavailable = nav.menu_unavailable.contains(&page_index);
        let advanced = nav.simple && nav::ADVANCED_PAGES.contains(&page.key);
        let note = if unavailable { "Not used with your current settings." } else { page.sentence };
        let strength = if unavailable { 0.45 } else { 1.0 } * ease;
        let number = typed(
            format!("{:02}", position + 1),
            MONO_FONT,
            10.0,
            scale,
            with_alpha(
                if here { palette.primary } else { palette.surface_text },
                if here { ease } else { 0.36 * ease },
            ),
        );
        let glyph = container(icon(page.icon, 14.0, scale, with_alpha(palette.primary, strength)))
            .width(Length::Fixed(20.0 * scale));
        let name = typed(
            page.label,
            TEXT_STRONG_FONT,
            13.0,
            scale,
            with_alpha(palette.surface_text, strength),
        );
        let sentence =
            typed(note, TEXT_FONT, 11.0, scale, with_alpha(palette.surface_text, 0.54 * ease))
                .line_height(iced::widget::text::LineHeight::Relative(1.35))
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph);
        let tag = advanced.then(|| {
            container(typed(
                "Advanced",
                MONO_FONT,
                8.5,
                scale,
                with_alpha(palette.surface_text, 0.55 * ease),
            ))
            .padding([1.0 * scale, 5.0 * scale])
            .style(move |_| crate::frontend::ui::tag_style(palette, ease, false))
        });
        let picture = nav.menu_picture.as_ref().filter(|(owner, _)| *owner == page_index).map(
            |(_, handle)| {
                container(
                    image(handle.clone())
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .content_fit(iced::ContentFit::Cover)
                        .opacity(ease),
                )
                .width(Length::Fixed(if grid { item_width - 22.0 * scale } else { 72.0 * scale }))
                .height(Length::Fixed(if grid { 62.0 } else { 40.5 } * scale))
                .clip(true)
            },
        );
        let body: Element<'a, Message> = if grid {
            let mut body = column![
                row![number, spacer(), glyph].align_y(Alignment::Center),
                row![name].push_maybe(tag).spacing(6.0 * scale).align_y(Alignment::Center),
                sentence,
            ]
            .spacing(6.0 * scale);
            if let Some(picture) = picture {
                body = body.push(picture);
            }
            body.into()
        } else {
            row![
                container(number).padding(Padding { top: 2.0 * scale, ..Padding::ZERO }),
                glyph.padding(Padding { top: 1.0 * scale, ..Padding::ZERO }),
                column![
                    row![name].push_maybe(tag).spacing(6.0 * scale).align_y(Alignment::Center),
                    sentence
                ]
                .spacing(3.0 * scale)
                .width(Length::Fill),
            ]
            .push_maybe(picture)
            .spacing(10.0 * scale)
            .align_y(Alignment::Start)
            .into()
        };
        let message = if nav.edit_cards && nav.page.is_none() {
            SettingsMsg::CardAdd(page_index)
        } else {
            SettingsMsg::OpenPage(page_index)
        };
        let mut item = button(body)
            .width(Length::Fixed(item_width))
            .padding([10.0 * scale, 11.0 * scale])
            .on_press(Message::Settings(message))
            .style(move |_theme, status| {
                crate::frontend::ui::tile_button_style(palette, ease, selected, here, true, status)
            });
        if grid {
            item = item.height(Length::Fixed(118.0 * scale));
        }
        current = current.push(item);
        in_row += 1;
        if in_row == columns || position + 1 == pages.len() {
            lines = lines.push(std::mem::replace(&mut current, row![].spacing(gap)));
            in_row = 0;
        }
    }
    let group_info = &GROUPS[group];
    let header = row![
        typed(group_info.label, DISPLAY_FONT, 13.0, scale, with_alpha(palette.surface_text, ease)),
        spacer(),
        typed(
            format!("{} pages", pages.len()),
            MONO_FONT,
            9.5,
            scale,
            with_alpha(palette.surface_text, 0.36 * ease)
        ),
    ]
    .align_y(Alignment::Center);
    let footer = typed(
        if nav.edit_cards && nav.page.is_none() {
            "Click a page to pin it on Home."
        } else {
            "Arrow keys choose, Enter opens, Esc closes"
        },
        MONO_FONT,
        9.5,
        scale,
        with_alpha(palette.surface_text, 0.34 * ease),
    );
    let panel_width = item_width * columns as f32 + gap * (columns as f32 - 1.0) + 24.0 * scale;
    let panel = container(column![header, lines, footer].spacing(10.0 * scale))
        .width(Length::Fixed(panel_width))
        .padding(12.0 * scale)
        .style(move |_| crate::frontend::ui::floating_panel_style(palette, ease));
    let slide = if nav.still { 0.0 } else { 8.0 * (1.0 - ease) * scale };
    mouse_area(container(panel).padding(Padding { top: slide, ..Padding::ZERO }))
        .on_enter(Message::Settings(SettingsMsg::MenuHover(group)))
        .on_exit(Message::Settings(SettingsMsg::MenuLeave))
        .into()
}

/// "← Back   Motion / Transitions / Combine two".
pub(super) fn breadcrumb<'a>(
    nav: &NavView<'a>,
    page: usize,
    segment: Option<String>,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    let info = &PAGES[page];
    let link = |content: Element<'a, Message>, message: SettingsMsg, active: bool| {
        button(content)
            .padding([3.0 * scale, 5.0 * scale])
            .on_press(Message::Settings(message))
            .style(move |_theme, status| quiet(palette, fade, active, false, status))
    };
    let sep = || typed("/", MONO_FONT, 11.0, scale, with_alpha(palette.surface_text, 0.26 * fade));
    let back = link(
        row![
            text(ICON_BACK)
                .font(crate::frontend::ui::NERD_FONT)
                .size(12.0 * legible_type_scale(scale)),
            plain(if nav.can_go_back { "Back" } else { "Home" }, MONO_FONT, 11.0, scale),
        ]
        .spacing(6.0 * scale)
        .align_y(Alignment::Center)
        .into(),
        SettingsMsg::Back,
        false,
    );
    let mut trail = row![
        back,
        container(text("")).width(Length::Fixed(12.0 * scale)),
        link(
            plain(GROUPS[info.group].label, MONO_FONT, 11.0, scale).into(),
            SettingsMsg::MenuClick(info.group),
            false
        ),
        sep(),
        link(
            plain(info.label, MONO_FONT, 11.0, scale).into(),
            SettingsMsg::OpenPage(page),
            segment.is_none()
        ),
    ]
    .spacing(2.0 * scale)
    .align_y(Alignment::Center);
    if let Some(segment) = segment {
        trail = trail.push(sep());
        trail = trail.push(
            container(typed(
                segment,
                MONO_FONT,
                11.0,
                scale,
                with_alpha(palette.surface_text, fade),
            ))
            .padding([3.0 * scale, 5.0 * scale]),
        );
    }
    trail.into()
}

/// The page under the bar: where you are, a big title with a short caption
/// on the right, a thin rule, then the content.
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
        10.0 * (1.0 - crate::frontend::animation::smoothstep(nav.page_fade)) * scale
    };
    let page_fade = fade * if nav.still { 1.0 } else { 0.2 + 0.8 * nav.page_fade.clamp(0.0, 1.0) };
    let title = text(info.label)
        .font(DISPLAY_FONT)
        .size(30.0 * scale.clamp(0.85, 1.15))
        .color(with_alpha(palette.surface_text, page_fade));
    let caption = container(
        typed(
            info.sentence,
            TEXT_FONT,
            12.5,
            scale,
            with_alpha(palette.surface_text, 0.56 * page_fade),
        )
        .line_height(iced::widget::text::LineHeight::Relative(1.45))
        .align_x(iced::alignment::Horizontal::Right)
        .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
    )
    .width(Length::Fixed((width * 0.34).min(300.0 * scale)))
    .align_x(crate::frontend::ui::end());
    let mut crumbs = row![breadcrumb(nav, page, segment, scale, palette, page_fade), spacer()]
        .spacing(8.0 * scale)
        .align_y(Alignment::Center);
    if hidden > 0 {
        crumbs = crumbs.push(typed(
            format!(
                "Simple view hides {hidden} advanced setting{} here",
                if hidden == 1 { "" } else { "s" }
            ),
            MONO_FONT,
            10.5,
            scale,
            with_alpha(palette.surface_text, 0.4 * page_fade),
        ));
        crumbs = crumbs.push(
            button(plain("Show them", MONO_FONT, 10.5, scale))
                .padding([3.0 * scale, 5.0 * scale])
                .on_press(Message::Settings(SettingsMsg::ShowAllRows))
                .style(move |_theme, status| quiet(palette, page_fade, true, false, status)),
        );
    }
    let header = column![
        crumbs,
        row![title, spacer(), caption].align_y(Alignment::End),
        rule(palette, 0.2 * page_fade),
    ]
    .spacing(12.0 * scale);
    let body = column![header, content].spacing(16.0 * scale).width(Length::Fixed(width));
    let padded = container(body).width(Length::Fill).center_x(Length::Fill).padding(Padding {
        top: 12.0 * scale + slide,
        right: 24.0 * scale,
        bottom: 72.0 * scale,
        left: 24.0 * scale,
    });
    scrollable(padded)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new().width(3.0 * scale).scroller_width(3.0 * scale),
        ))
        .id(crate::frontend::ui::pane_id(super::folio::READING_PANE))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// The command palette, over everything: one quiet field and a list of
/// results; values change in place.
pub(super) fn palette_layer<'a>(
    view: &PaletteView<'a>,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    let input = text_input("Find a setting, page, transition, layout or action", view.query)
        .id(super::super::palette_input_id())
        .font(TEXT_FONT)
        .on_input(|value| Message::Settings(SettingsMsg::PaletteInput(value)))
        .on_submit(Message::Settings(SettingsMsg::PaletteSubmit))
        .size(15.0 * legible_type_scale(scale))
        .padding([10.0 * scale, 12.0 * scale])
        .width(Length::Fill)
        .style(move |_theme, status| crate::frontend::ui::quiet_input_style(palette, fade, status));
    let mut list = column![].spacing(2.0 * scale);
    if view.empty_query {
        list = list.push(
            container(typed(
                "Recent and suggested",
                MONO_FONT,
                10.0,
                scale,
                with_alpha(palette.surface_text, 0.42 * fade),
            ))
            .padding([2.0 * scale, 10.0 * scale]),
        );
    }
    if view.results.is_empty() {
        list = list.push(
            container(typed(
                "Nothing found. Try fewer letters or another word.",
                TEXT_FONT,
                12.0,
                scale,
                with_alpha(palette.surface_text, 0.6 * fade),
            ))
            .padding([14.0 * scale, 10.0 * scale]),
        );
    }
    for (position, entry) in view.results.iter().enumerate() {
        let selected = position == view.selected;
        let value: Element<'a, Message> = match &entry.value {
            PaletteValue::Toggle { value, .. } => {
                let on = *value;
                button(
                    text(if on { "On" } else { "Off" })
                        .font(MONO_FONT)
                        .size(10.5 * legible_type_scale(scale)),
                )
                .padding([3.0 * scale, 9.0 * scale])
                .on_press(Message::Settings(SettingsMsg::PaletteChange(position, None)))
                .style(move |_theme, status| {
                    crate::frontend::ui::filter_button_style(palette, fade, on, status)
                })
                .into()
            }
            PaletteValue::Choice { options, current, .. } => {
                let mut chips = row![].spacing(2.0 * scale).align_y(Alignment::Center);
                for (key, text_) in options.iter().take(5) {
                    let on = key == current;
                    chips = chips.push(
                        button(
                            text(text_.clone())
                                .font(MONO_FONT)
                                .size(10.0 * legible_type_scale(scale)),
                        )
                        .padding([3.0 * scale, 7.0 * scale])
                        .on_press(Message::Settings(SettingsMsg::PaletteChange(
                            position,
                            Some(key.clone()),
                        )))
                        .style(move |_theme, status| {
                            crate::frontend::ui::filter_button_style(palette, fade, on, status)
                        }),
                    );
                }
                if options.len() > 5 {
                    chips = chips.push(typed(
                        format!("+{}", options.len() - 5),
                        MONO_FONT,
                        10.0,
                        scale,
                        with_alpha(palette.surface_text, 0.42 * fade),
                    ));
                }
                chips.into()
            }
            PaletteValue::None => typed(
                entry.kind.label(),
                MONO_FONT,
                9.5,
                scale,
                with_alpha(palette.surface_text, 0.38 * fade),
            )
            .into(),
        };
        let copy = column![
            text(entry.title.clone()).font(TEXT_STRONG_FONT).size(13.0 * legible_type_scale(scale)),
            typed(
                entry.crumb.clone(),
                MONO_FONT,
                9.5,
                scale,
                with_alpha(palette.surface_text, 0.42 * fade)
            ),
        ]
        .spacing(2.0 * scale)
        .width(Length::Fill);
        let item = button(
            row![
                container(icon(entry.icon, 14.0, scale, with_alpha(palette.primary, 0.9 * fade)))
                    .width(Length::Fixed(22.0 * scale)),
                copy,
                value,
            ]
            .spacing(10.0 * scale)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([7.0 * scale, 10.0 * scale])
        .on_press(Message::Settings(SettingsMsg::PaletteRun(position)))
        .style(move |_theme, status| quiet(palette, fade, selected, selected, status));
        list = list.push(
            mouse_area(item).on_enter(Message::Settings(SettingsMsg::PaletteHover(position))),
        );
    }
    let hints = typed(
        "Arrow keys choose, Tab changes it here, Enter goes there, Esc closes",
        MONO_FONT,
        9.5,
        scale,
        with_alpha(palette.surface_text, 0.34 * fade),
    );
    let card = container(
        column![input, list, container(hints).padding([0.0, 10.0 * scale])].spacing(10.0 * scale),
    )
    .width(Length::Fixed(640.0 * scale))
    .padding(12.0 * scale)
    .style(move |_| crate::frontend::ui::floating_panel_style(palette, fade));
    let scrim = mouse_area(
        container(text(""))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| crate::frontend::ui::ink_fill(palette.surface, 0.72 * fade)),
    )
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

/// "Changed X to Y" with Undo, at the bottom of the window.
pub(super) fn toast_layer(
    text_: String,
    can_undo: bool,
    scale: f32,
    palette: &Palette,
    fade: f32,
) -> Element<'_, Message> {
    let ease = crate::frontend::animation::smoothstep(fade.clamp(0.0, 1.0));
    let mut content =
        row![typed(text_, TEXT_FONT, 12.0, scale, with_alpha(palette.surface_text, ease))]
            .spacing(12.0 * scale)
            .align_y(Alignment::Center);
    if can_undo {
        content = content.push(
            button(
                row![
                    text("Undo").font(MONO_FONT).size(10.5 * legible_type_scale(scale)),
                    typed(
                        "Ctrl Z",
                        MONO_FONT,
                        9.5,
                        scale,
                        with_alpha(palette.primary_text, 0.6 * ease)
                    ),
                ]
                .spacing(7.0 * scale)
                .align_y(Alignment::Center),
            )
            .padding([4.0 * scale, 9.0 * scale])
            .on_press(Message::Settings(SettingsMsg::Undo))
            .style(move |_theme, status| {
                crate::frontend::ui::tile_action_style(palette, ease, true, false, status)
            }),
        );
    }
    let note = container(content)
        .padding(Padding {
            top: 7.0 * scale,
            right: 8.0 * scale,
            bottom: 7.0 * scale,
            left: 13.0 * scale,
        })
        .style(move |_| crate::frontend::ui::floating_panel_style(palette, ease));
    container(iced::widget::opaque(note))
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center)
        .align_y(Alignment::End)
        .padding(Padding { bottom: 18.0 * scale + 8.0 * (1.0 - ease) * scale, ..Padding::ZERO })
        .into()
}

/// "Preview big": the one live preview, large, over a blurred and darkened
/// copy of the background. Esc or a click beside it closes it.
pub(super) fn big_preview_layer<'a>(
    big: &BigPreview,
    blurred: Option<iced::advanced::image::Handle>,
    panel: (f32, f32),
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    let ease = crate::frontend::animation::smoothstep(big.fade.clamp(0.0, 1.0)) * fade;
    let width = (panel.0 * 0.66).min((panel.1 - 190.0 * scale) * 16.0 / 9.0).max(320.0 * scale);
    let height = width * 9.0 / 16.0;
    let frame: Element<'a, Message> = match big.frame.clone() {
        Some(handle) => image(handle)
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(iced::ContentFit::Cover)
            .opacity(ease)
            .into(),
        None => container(typed(
            "starting the preview",
            MONO_FONT,
            10.5,
            scale,
            with_alpha(palette.surface_text, 0.4 * ease),
        ))
        .center(Length::Fill)
        .into(),
    };
    let stage = container(frame)
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .clip(true)
        .style(move |_| crate::frontend::ui::ink_fill(palette.surface_variant, ease));
    let caption = row![
        column![
            typed(
                big.title.clone(),
                DISPLAY_FONT,
                15.0,
                scale,
                with_alpha(palette.surface_text, ease)
            ),
            typed(
                big.desc.clone(),
                TEXT_FONT,
                12.0,
                scale,
                with_alpha(palette.surface_text, 0.6 * ease)
            )
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
        ]
        .spacing(5.0 * scale)
        .width(Length::Fill),
        button(
            row![
                plain("Close", MONO_FONT, 11.0, scale),
                typed("Esc", MONO_FONT, 10.0, scale, with_alpha(palette.surface_text, 0.36 * ease)),
            ]
            .spacing(7.0 * scale)
            .align_y(Alignment::Center)
        )
        .padding([5.0 * scale, 8.0 * scale])
        .on_press(Message::Settings(SettingsMsg::BigPreviewClose))
        .style(move |_theme, status| quiet(palette, ease, false, false, status)),
    ]
    .spacing(12.0 * scale)
    .align_y(Alignment::Start)
    .width(Length::Fixed(width));
    let card = container(column![stage, caption].spacing(12.0 * scale))
        .padding(12.0 * scale)
        .style(move |_| crate::frontend::ui::floating_panel_style(palette, ease));
    let blur: Element<'a, Message> = match blurred {
        Some(handle) => image(handle)
            .width(Length::Fill)
            .height(Length::Fill)
            .content_fit(iced::ContentFit::Cover)
            .opacity(0.55 * ease)
            .into(),
        None => iced::widget::Space::new().width(Length::Fill).height(Length::Fill).into(),
    };
    let veil = mouse_area(
        container(text(""))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| crate::frontend::ui::ink_fill(palette.surface, 0.8 * ease)),
    )
    .on_press(Message::Settings(SettingsMsg::BigPreviewClose));
    stack![
        iced::widget::opaque(blur),
        veil,
        container(iced::widget::opaque(card)).center(Length::Fill),
    ]
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
