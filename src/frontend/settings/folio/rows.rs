//! R1 (custom build): one row style for every page: name and one plain
//! sentence on the left, the control on the right, a dot and a reset button
//! when the value is not the default, "more…" for a longer explanation, and
//! cards (see `cards.rs`) where a row is a card. One column only.

use std::collections::{HashMap, HashSet};

use iced::widget::{button, column, container, mouse_area, text};
use iced::{Alignment, Element, Length, Padding};

use crate::app::Message;
use crate::contracts::settings::{SettingsSource, schema};
use crate::frontend::animation::MotionProfile;
use crate::frontend::components::with_alpha;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{TEXT_FONT, TEXT_STRONG_FONT, label, legible_type_scale, row, typed};

use super::super::nav::{self, ICON_RESET};
use super::super::{Control, Row, SettingsFocus, SettingsMsg};
use super::folio::FocusCtx;

pub(super) struct RowsInput<'a> {
    pub rows: Vec<Row>,
    pub values: &'a HashMap<String, String>,
    pub has_selected_preset: bool,
    pub width: f32,
    pub scale: f32,
    pub palette: &'a Palette,
    pub fade: f32,
    pub motion: MotionProfile,
    pub cfg: &'a dyn SettingsSource,
    pub more: &'a HashSet<String>,
    /// R1b: what the cards show (pictures, the lit card).
    pub pictures: &'a HashMap<String, iced::advanced::image::Handle>,
    pub playing: Option<(&'a str, &'a iced::advanced::image::Handle)>,
    pub tile_hover: Option<(usize, usize)>,
    pub tile_fade: f32,
    pub follow_cursor: bool,
    pub cursor: Option<super::cards::CursorView<'a>>,
    /// Card pictures the background renderer still has to draw.
    pub pending: usize,
    pub card_layout: super::cards::CardLayout,
}

/// Rows next to a card grid stay a comfortable reading width.
const ROW_MAX_WIDTH: f32 = 800.0;

/// The search field and the filter of a card grid (drawn as one line).
fn toolbar_search(control: &Control) -> Option<(String, &'static str)> {
    match control {
        Control::TextField { key, placeholder, .. }
            if key == crate::contracts::settings::keys::transition::LIBRARY_SEARCH =>
        {
            Some((key.clone(), *placeholder))
        }
        _ => None,
    }
}

fn toolbar_filter(control: &Control) -> Option<(String, Vec<(String, String)>, String)> {
    match control {
        Control::Chips { path, options, current, .. }
            if path == crate::contracts::settings::keys::transition::LIBRARY_FILTER =>
        {
            Some((path.clone(), options.clone(), current.clone()))
        }
        _ => None,
    }
}

/// Whether `control` shows a value that differs from the schema default.
pub(super) fn differs_from_default(control: &Control, cfg: &dyn SettingsSource) -> Option<String> {
    let path = nav_path(control)?;
    let differs = match control {
        Control::Toggle { value, .. } => {
            schema::boolean_default(&path).map(|default| *value != default)
        }
        Control::Number { .. } => {
            schema::number_default(&path).map(|default| (cfg.number(&path) - default).abs() > 1e-6)
        }
        Control::Chips { current, .. } | Control::Dropdown { current, .. } => {
            match schema::value_kind(&path) {
                Some(schema::ValueKind::Number) => schema::number_default(&path).map(|default| {
                    current.parse::<f64>().is_ok_and(|value| (value - default).abs() > 1e-6)
                }),
                Some(schema::ValueKind::Boolean) => {
                    schema::boolean_default(&path).map(|default| (current == "true") != default)
                }
                _ => schema::text_default(&path).map(|default| current != default),
            }
        }
        Control::TextField { .. } | Control::KeyBinding { .. } => {
            schema::text_default(&path).map(|default| cfg.text(&path) != default)
        }
        _ => None,
    };
    differs.filter(|differs| *differs).map(|_| path)
}

fn nav_path(control: &Control) -> Option<String> {
    match control {
        Control::Toggle { path, .. }
        | Control::Number { path, .. }
        | Control::TextField { path, .. }
        | Control::KeyBinding { path, .. }
        | Control::Dropdown { path, .. }
        | Control::Chips { path, .. } => Some(path.clone()),
        _ => None,
    }
}

/// Rows that keep the old full-width editors (groups, folds, lists).
fn wide(control: &Control) -> bool {
    matches!(
        control,
        Control::Segment
            | Control::Details { .. }
            | Control::StackBar { .. }
            | Control::MotionWeights { .. }
            | Control::AppTheme { .. }
            | Control::Presets { .. }
            | Control::Code { .. }
            | Control::Display(_)
            | Control::Resolution { .. }
    )
}

/// Share of the row the control gets.
fn control_share(control: &Control) -> f32 {
    match control {
        Control::Toggle { .. }
        | Control::ToggleAction { .. }
        | Control::ActionBtn { .. }
        | Control::Number { .. } => 0.3,
        Control::Chips { options, .. } | Control::Dropdown { options, .. } if options.len() > 4 => {
            0.6
        }
        Control::ActionChips { items } if items.len() > 2 => 0.62,
        _ => 0.5,
    }
}

pub(super) fn rows<'a>(input: RowsInput<'a>, focus: FocusCtx<'_>) -> Element<'a, Message> {
    let RowsInput {
        rows,
        values,
        has_selected_preset,
        width,
        scale,
        palette,
        fade,
        motion,
        cfg,
        more,
        pictures,
        playing,
        tile_hover,
        tile_fade,
        follow_cursor,
        cursor,
        pending,
        card_layout,
    } = input;
    let keyboard_row =
        (focus.keyboard_focus == SettingsFocus::Controls).then_some(focus.focused_control);
    let tile_ctx = super::cards::TileCtx {
        palette,
        scale,
        fade,
        width,
        pictures,
        playing,
        hover: tile_hover,
        hover_fade: tile_fade,
        follow_cursor,
        cursor,
        focus: keyboard_row,
        choice: focus.focused_choice,
        first_number: 1,
        layout: card_layout,
    };
    let card_count = rows.iter().filter(|row| matches!(row.control, Control::Tile { .. })).count();
    let gallery_width = width;
    let lit_note = tile_hover
        .map(|(row, _)| row)
        .or(keyboard_row)
        .and_then(|index| rows.get(index))
        .filter(|row| matches!(row.control, Control::Tile { .. }))
        .map(|row| format!("{}: {}", row.title, row.desc));
    let row_width = width.min(ROW_MAX_WIDTH * scale);
    let mut run: Vec<super::cards::TileItem> = Vec::new();
    let mut search: Option<(usize, (String, &'static str))> = None;
    if rows.is_empty() {
        return label(
            crate::i18n::tr("settings-section-empty"),
            12.0,
            scale,
            with_alpha(palette.surface_text, 0.56 * fade),
        )
        .into();
    }
    let mut list = column![].width(Length::Fill);
    for (index, setting) in rows.into_iter().enumerate() {
        if matches!(setting.control, Control::Preview) {
            continue;
        }
        if matches!(setting.control, Control::Tile { .. }) {
            run.extend(super::cards::TileItem::from_row(index, setting));
            continue;
        }
        if !run.is_empty() {
            list = list.push(super::cards::tile_grid(std::mem::take(&mut run), tile_ctx));
        }
        if let Some(field) = toolbar_search(&setting.control) {
            search = Some((index, field));
            continue;
        }
        if let Some(filter) = toolbar_filter(&setting.control) {
            let found = search.take();
            let note = lit_note.clone().unwrap_or_else(|| {
                if pending > 0 {
                    format!("{card_count} shown. Drawing their pictures once, {pending} to go.")
                } else {
                    format!("{card_count} shown. Point at a card to see it play.")
                }
            });
            list = list.push(
                container(super::cards::toolbar(
                    super::cards::Toolbar {
                        search_focus: found.as_ref().map(|(row, _)| *row) == keyboard_row
                            && keyboard_row.is_some(),
                        search: found.map(|(_, field)| field),
                        values,
                        filter: Some(filter),
                        filter_focused: keyboard_row == Some(index),
                        filter_choice: focus.focused_choice,
                        note,
                    },
                    tile_ctx,
                ))
                .padding(Padding {
                    top: 2.0 * scale,
                    bottom: 6.0 * scale,
                    ..Padding::ZERO
                }),
            );
            continue;
        }
        if nav::gallery(&setting.control).is_some() {
            list = list.push(gallery_row(setting, index, gallery_width, tile_ctx, focus, cfg));
            continue;
        }
        let width = row_width;
        if matches!(setting.control, Control::Segment) {
            // The first heading of a page of cards says what the page title
            // already says.
            if index == 0 && card_count > 0 {
                continue;
            }
            list = list.push(segment_heading(setting, index == 0, width, scale, palette, fade));
            continue;
        }
        let item: Element<'a, Message> = if wide(&setting.control) {
            let top_pad = if matches!(setting.control, Control::Segment) { 18.0 } else { 8.0 };
            container(super::folio::field(
                setting,
                index,
                values,
                has_selected_preset,
                focus,
                width,
                scale,
                palette,
                fade,
                motion,
            ))
            .padding(Padding { top: top_pad * scale, bottom: 8.0 * scale, ..Padding::ZERO })
            .width(Length::Fixed(width))
            .into()
        } else {
            line(
                setting,
                index,
                values,
                has_selected_preset,
                focus,
                width,
                scale,
                palette,
                fade,
                motion,
                cfg,
                more,
            )
        };
        list = list.push(container(item).width(Length::Fixed(width)));
    }
    if !run.is_empty() {
        list = list.push(super::cards::tile_grid(run, tile_ctx));
    }
    list.into()
}

/// A setting whose choices are cards: its name and sentence (with the
/// changed dot and reset), then the grid.
fn gallery_row<'a>(
    setting: Row,
    index: usize,
    width: f32,
    ctx: super::cards::TileCtx<'a>,
    focus: FocusCtx<'_>,
    cfg: &dyn SettingsSource,
) -> Element<'a, Message> {
    let super::cards::TileCtx { palette, scale, fade, .. } = ctx;
    let changed = differs_from_default(&setting.control, cfg);
    let focused = focus.keyboard_focus == SettingsFocus::Controls && focus.focused_control == index;
    let Some(gallery) = nav::gallery(&setting.control) else {
        return text("").into();
    };
    let items: Vec<super::cards::ChoiceItem> = gallery
        .options
        .iter()
        .map(|(key, name)| super::cards::ChoiceItem {
            key: key.clone(),
            name: name.clone(),
            picture: gallery.picture(key, cfg),
            current: key == gallery.current,
            disabled: gallery.disabled.contains(key),
        })
        .collect();
    let path = gallery.path.to_string();
    let previews = gallery.previews();
    let dot = typed(
        "●",
        TEXT_FONT,
        6.0,
        scale,
        with_alpha(palette.primary, if changed.is_some() { fade } else { 0.0 }),
    );
    let mut copy = column![
        row![
            typed(
                setting.title,
                TEXT_STRONG_FONT,
                13.5,
                scale,
                with_alpha(palette.surface_text, fade)
            ),
            container(dot).padding(Padding { top: 5.0 * scale, ..Padding::ZERO }),
        ]
        .spacing(7.0 * scale)
        .align_y(Alignment::Start)
    ]
    .spacing(4.0 * scale)
    .width(Length::Fill);
    if !setting.desc.trim().is_empty() {
        copy = copy.push(
            typed(
                setting.desc,
                TEXT_FONT,
                11.5,
                scale,
                with_alpha(palette.surface_text, 0.54 * fade),
            )
            .line_height(iced::widget::text::LineHeight::Relative(1.42))
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
        );
    }
    let reset: Element<'a, Message> = match changed {
        Some(path) => button(
            text(ICON_RESET)
                .font(crate::frontend::ui::NERD_FONT)
                .size(12.0 * legible_type_scale(scale)),
        )
        .padding([3.0 * scale, 5.0 * scale])
        .on_press(Message::Settings(SettingsMsg::ResetPath(path)))
        .style(move |_theme, status| {
            crate::frontend::ui::text_button_style(palette, fade, false, false, status)
        })
        .into(),
        None => text("").into(),
    };
    let ring = focused && focus.focused_choice.is_none();
    let heading = mouse_area(
        container(row![copy, reset].spacing(8.0 * scale).align_y(Alignment::Center))
            .width(Length::Fixed(width.min(ROW_MAX_WIDTH * scale)))
            .padding(Padding {
                top: 13.0 * scale,
                right: 4.0 * scale,
                bottom: 11.0 * scale,
                left: 10.0 * scale,
            })
            .style(move |_| crate::frontend::ui::plain_row_style(palette, fade, ring)),
    )
    .on_press(Message::Settings(SettingsMsg::FocusControl(index)));
    column![
        heading,
        super::cards::choice_grid(index, &path, items, previews, ctx),
        container(super::shell::rule(palette, 0.1 * fade))
            .padding(Padding { top: 16.0 * scale, ..Padding::ZERO }),
    ]
    .spacing(4.0 * scale)
    .into()
}

/// A small heading over a group of rows: its name and one sentence.
fn segment_heading(
    setting: Row,
    first: bool,
    width: f32,
    scale: f32,
    palette: &Palette,
    fade: f32,
) -> Element<'_, Message> {
    let mut copy = column![typed(
        setting.title,
        TEXT_STRONG_FONT,
        15.0,
        scale,
        with_alpha(palette.surface_text, fade)
    )]
    .spacing(4.0 * scale);
    if !setting.desc.trim().is_empty() {
        copy = copy.push(
            typed(
                setting.desc,
                TEXT_FONT,
                11.5,
                scale,
                with_alpha(palette.surface_text, 0.52 * fade),
            )
            .line_height(iced::widget::text::LineHeight::Relative(1.4))
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
        );
    }
    container(copy)
        .width(Length::Fixed(width))
        .padding(Padding {
            top: if first { 0.0 } else { 30.0 * scale },
            bottom: 8.0 * scale,
            ..Padding::ZERO
        })
        .into()
}

#[allow(clippy::too_many_arguments)]
fn line<'a>(
    setting: Row,
    index: usize,
    values: &'a HashMap<String, String>,
    has_selected_preset: bool,
    focus: FocusCtx<'_>,
    width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
    cfg: &dyn SettingsSource,
    more: &HashSet<String>,
) -> Element<'a, Message> {
    let Row { title, desc, control } = setting;
    let focused = focus.keyboard_focus == SettingsFocus::Controls && focus.focused_control == index;
    let choice = if focused { focus.focused_choice } else { None };
    let changed = differs_from_default(&control, cfg);
    let path = nav_path(&control).unwrap_or_default();
    let explanation = nav::more_text(&path);
    let more_id = format!("{index}:{path}");
    let open_more = explanation.is_some() && more.contains(&more_id);
    let control_width = (width * control_share(&control)).max(150.0 * scale);
    // Room for the dot, the gaps and the reset button.
    let copy_width = (width - control_width - 76.0 * scale).max(120.0 * scale);
    let dot = typed(
        "●",
        TEXT_FONT,
        6.0,
        scale,
        with_alpha(palette.primary, if changed.is_some() { fade } else { 0.0 }),
    );
    let name = row![
        typed(title, TEXT_STRONG_FONT, 13.5, scale, with_alpha(palette.surface_text, fade))
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
        container(dot).padding(Padding { top: 5.0 * scale, ..Padding::ZERO }),
    ]
    .spacing(7.0 * scale)
    .align_y(Alignment::Start);
    let mut copy = column![name].spacing(4.0 * scale).width(Length::Fixed(copy_width));
    if !desc.trim().is_empty() {
        copy = copy.push(
            typed(desc, TEXT_FONT, 11.5, scale, with_alpha(palette.surface_text, 0.54 * fade))
                .line_height(iced::widget::text::LineHeight::Relative(1.42))
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
        );
    }
    if let Some(explanation) = explanation {
        let toggle = button(
            text(if open_more { "less" } else { "more…" })
                .font(crate::frontend::ui::MONO_FONT)
                .size(10.5 * legible_type_scale(scale)),
        )
        .padding([1.0 * scale, 0.0])
        .on_press(Message::Settings(SettingsMsg::ToggleMore(more_id)))
        .style(move |_theme, status| {
            crate::frontend::ui::text_button_style(palette, fade, true, false, status)
        });
        copy = copy.push(toggle);
        if open_more {
            copy = copy.push(
                typed(
                    explanation,
                    TEXT_FONT,
                    11.5,
                    scale,
                    with_alpha(palette.surface_text, 0.76 * fade),
                )
                .line_height(iced::widget::text::LineHeight::Relative(1.45)),
            );
        }
    }
    let editor = super::control::widget(
        control,
        values,
        has_selected_preset,
        focused,
        choice,
        control_width,
        scale,
        palette,
        fade,
        focus.armed,
        motion,
        Some(focus.reveals()),
    );
    let reset: Element<'a, Message> = match changed {
        Some(path) => button(
            text(ICON_RESET)
                .font(crate::frontend::ui::NERD_FONT)
                .size(12.0 * legible_type_scale(scale)),
        )
        .padding([3.0 * scale, 5.0 * scale])
        .on_press(Message::Settings(SettingsMsg::ResetPath(path)))
        .style(move |_theme, status| {
            crate::frontend::ui::text_button_style(palette, fade, false, false, status)
        })
        .into(),
        None => container(text("")).width(Length::Fixed(22.0 * scale)).into(),
    };
    let body = row![
        copy,
        container(text("")).width(Length::Fill),
        container(editor).width(Length::Fixed(control_width)).align_x(crate::frontend::ui::end()),
        reset,
    ]
    .spacing(8.0 * scale)
    .align_y(Alignment::Center);
    let highlight = focused;
    column![
        mouse_area(
            container(body)
                .width(Length::Fill)
                .padding(Padding {
                    top: 13.0 * scale,
                    right: 4.0 * scale,
                    bottom: 13.0 * scale,
                    left: 10.0 * scale,
                })
                .style(move |_| crate::frontend::ui::plain_row_style(palette, fade, highlight)),
        )
        .on_press(Message::Settings(SettingsMsg::FocusControl(index))),
        super::shell::rule(palette, 0.1 * fade),
    ]
    .into()
}
