use std::collections::HashMap;

use iced::widget::{button, column, container, text, text_input};
use iced::{Alignment, Color, Element, Length};

use crate::app::Message;
use crate::frontend::animation::MotionProfile;
use crate::frontend::components::with_alpha;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{legible_type_scale, row};
use crate::i18n::tr;

use super::super::{ActionId, Control, PRESET_NAME_KEY, SettingsMsg};

const BUTTON_HEIGHT: f32 = 26.0;
const BUTTON_MIN_WIDTH: f32 = 48.0;

/// Small text in the Settings text face.
fn label<'a>(
    content: impl iced::widget::text::IntoFragment<'a>,
    size: f32,
    scale: f32,
    color: Color,
) -> iced::widget::Text<'a> {
    crate::frontend::ui::typed(content, crate::frontend::ui::TEXT_FONT, size, scale, color)
}

pub(super) fn button_width(label: &str, scale: f32) -> f32 {
    crate::frontend::ui::folio_action_width_in(label, scale, BUTTON_MIN_WIDTH, f32::INFINITY)
}

/// One word that is a button (a choice, an action). R1b: no box; the chosen
/// one gets a thin accent outline, the keyboard a ring.
pub(super) fn fixed_button(
    label: String,
    active: bool,
    destructive: bool,
    message: Option<Message>,
    focused: bool,
    scale: f32,
    palette: &Palette,
    fade: f32,
    width: f32,
    _motion: MotionProfile,
) -> Element<'_, Message> {
    let label = container(
        text(label)
            .font(crate::frontend::ui::TEXT_STRONG_FONT)
            .size(11.5 * legible_type_scale(scale))
            .wrapping(iced::widget::text::Wrapping::None),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(iced::alignment::Horizontal::Center)
    .align_y(iced::alignment::Vertical::Center);
    let hit_target = button(label)
        .width(Length::Fixed(width))
        .height(Length::Fixed(BUTTON_HEIGHT * scale.max(1.0)))
        .padding(0.0)
        .style(move |_theme, status| {
            crate::frontend::ui::choice_button_style(
                palette,
                fade,
                active,
                destructive,
                focused,
                status,
            )
        });
    if let Some(message) = message { hit_target.on_press(message) } else { hit_target }.into()
}

/// A button that does something (not a choice): a thin outline that fills
/// on hover; `strong` (asking to confirm) uses the accent.
#[allow(clippy::too_many_arguments)]
fn action_button(
    label: String,
    strong: bool,
    message: Message,
    focused: bool,
    scale: f32,
    palette: &Palette,
    fade: f32,
    width: f32,
) -> Element<'_, Message> {
    button(
        container(
            text(label)
                .font(crate::frontend::ui::TEXT_STRONG_FONT)
                .size(11.5 * legible_type_scale(scale))
                .wrapping(iced::widget::text::Wrapping::None),
        )
        .center(Length::Fill),
    )
    .width(Length::Fixed(width))
    .height(Length::Fixed(BUTTON_HEIGHT * scale.max(1.0)))
    .padding(0.0)
    .on_press(message)
    .style(move |_theme, status| {
        crate::frontend::ui::tile_action_style(palette, fade, strong, focused, status)
    })
    .into()
}

/// A switch with its state in words next to it.
fn switch<'a>(
    on: bool,
    words: (&'static str, &'static str),
    message: Message,
    focused: bool,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
) -> Element<'a, Message> {
    let knob = container(text(""))
        .width(Length::Fixed(10.0 * scale))
        .height(Length::Fixed(10.0 * scale))
        .style(move |_| crate::frontend::ui::switch_thumb_style(palette, fade, on));
    let track = container(knob)
        .width(Length::Fixed(30.0 * scale))
        .height(Length::Fixed(16.0 * scale))
        .padding([0.0, 3.0 * scale])
        .align_y(Alignment::Center)
        .align_x(if on { crate::frontend::ui::end() } else { crate::frontend::ui::start() })
        .style(move |_| crate::frontend::ui::switch_track_style(palette, fade, on, focused));
    let word = text(if on { words.0 } else { words.1 })
        .font(crate::frontend::ui::MONO_FONT)
        .size(10.5 * legible_type_scale(scale))
        .color(with_alpha(palette.surface_text, if on { 0.9 * fade } else { 0.46 * fade }));
    button(row![word, track].spacing(9.0 * scale).align_y(Alignment::Center))
        .padding([5.0 * scale, 6.0 * scale])
        .on_press(message)
        .style(move |_theme, status| {
            crate::frontend::ui::text_button_style(palette, fade, on, focused, status)
        })
        .into()
}

fn option_button(
    label: String,
    active: bool,
    destructive: bool,
    message: Message,
    focused: bool,
    scale: f32,
    palette: &Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'_, Message> {
    let width = button_width(&label, scale);
    fixed_button(
        label,
        active,
        destructive,
        Some(message),
        focused,
        scale,
        palette,
        fade,
        width,
        motion,
    )
}

#[allow(clippy::too_many_arguments)]
fn choice_buttons<'a>(
    path: &str,
    options: Vec<(String, String)>,
    current: &str,
    disabled: &[String],
    palettes: &[(String, Vec<String>)],
    keyboard_focused: bool,
    focused_choice: Option<usize>,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    let items = options
        .into_iter()
        .enumerate()
        .map(|(index, (key, label))| {
            let width = button_width(&label, scale);
            let enabled = !disabled.contains(&key);
            let active = key == current;
            let colors =
                palettes.iter().rev().find(|(name, _)| name == &key).map(|(_, colors)| colors);
            let message =
                enabled.then(|| Message::Settings(SettingsMsg::Pick(path.to_owned(), key)));
            let item = fixed_button(
                label,
                active,
                false,
                message,
                enabled && keyboard_focused && focused_choice == Some(index),
                scale,
                palette,
                fade,
                width,
                motion,
            );
            let item = if let Some(colors) = colors {
                let mut strip = row![].width(Length::Fixed(width));
                for color in colors {
                    let color =
                        crate::frontend::theme::parse_hex(color).unwrap_or(Color::TRANSPARENT);
                    strip = strip.push(
                        container(text(""))
                            .width(Length::Fill)
                            .height(Length::Fixed(10.0 * scale))
                            .style(move |_| {
                                crate::frontend::ui::bg_style(iced::Background::Color(color))
                            }),
                    );
                }
                column![item, strip].into()
            } else {
                item
            };
            (width, item)
        })
        .collect();
    wrapped_options(items, available_width, scale)
}

fn wrapped_options(
    items: Vec<(f32, Element<'_, Message>)>,
    available_width: f32,
    scale: f32,
) -> Element<'_, Message> {
    let lines = crate::frontend::ui::wrap_rows(items, available_width, 7.0 * scale);
    let mut group = column![].spacing(7.0 * scale);
    for line in lines {
        let mut line_row = row![].spacing(7.0 * scale).align_y(Alignment::Center);
        for item in line {
            line_row = line_row.push(item);
        }
        group = group.push(line_row);
    }
    group.into()
}

#[derive(Clone, Copy)]
pub(super) struct Reveals<'a> {
    pub(super) bars: &'a HashMap<String, crate::frontend::animation::Tween>,
    pub(super) folds: &'a std::collections::HashSet<String>,
}

pub(super) fn widget<'a>(
    control: Control,
    values: &'a HashMap<String, String>,
    has_selected_preset: bool,
    keyboard_focused: bool,
    focused_choice: Option<usize>,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    armed: Option<&ActionId>,
    motion: MotionProfile,
    reveals: Option<Reveals<'_>>,
) -> Element<'a, Message> {
    match control {
        Control::Display(control) => control.view(scale, palette),
        Control::AppTheme { app, .. } => {
            label(app.name, crate::frontend::ui::TYPE_SMALL, scale, palette.surface_text).into()
        }
        Control::Toggle { path, value } => switch(
            value,
            (tr("settings-app-themes-on"), tr("settings-app-themes-off")),
            Message::Settings(SettingsMsg::Toggle(path, !value)),
            keyboard_focused,
            scale,
            palette,
            fade,
        ),
        Control::ToggleAction { id, value } => switch(
            value,
            (tr("settings-app-themes-on"), tr("settings-app-themes-off")),
            Message::Settings(SettingsMsg::Run(id)),
            keyboard_focused,
            scale,
            palette,
            fade,
        ),
        Control::Number { key, unit, .. } => {
            let value = values.get(&key).map_or("", String::as_str);
            let automatic = key == crate::contracts::settings::keys::transition::FPS;
            let value = if automatic && value.parse::<f64>() == Ok(0.0) { "" } else { value };
            let placeholder = if automatic { tr("settings-transitions-rate-auto") } else { "0" };
            let message_key = key.clone();
            row![
                text_input(placeholder, value)
                    .id(super::super::workbench_input_id(&key))
                    .font(crate::frontend::ui::MONO_FONT)
                    .on_input(move |raw| Message::Settings(SettingsMsg::Input(
                        message_key.clone(),
                        raw,
                    )))
                    .on_submit(Message::Settings(SettingsMsg::Commit))
                    .size(13.0 * legible_type_scale(scale))
                    .padding([6.0 * scale, 9.0 * scale])
                    .width(Length::Fixed(88.0 * scale))
                    .style(move |_theme, status| {
                        crate::frontend::ui::quiet_input_style(palette, fade, status)
                    }),
                label(
                    unit,
                    crate::frontend::ui::TYPE_SMALL,
                    scale,
                    with_alpha(palette.surface_text, 0.56 * fade)
                ),
            ]
            .spacing(8.0 * scale)
            .align_y(Alignment::Center)
            .into()
        }
        Control::KeyBinding { path, default, .. } => {
            let value = values.get(&path).map_or("", String::as_str);
            let triggers = crate::domain::input::parse_binding(value)
                .or_else(|| crate::domain::input::parse_binding(default))
                .unwrap_or_default();
            let pal = *palette;
            let mut slots = row![].spacing(6.0 * scale);
            for slot in 0..crate::domain::input::SLOTS {
                let shown = crate::domain::input::slot_triggers(&triggers, slot);
                let (display, alpha) = if shown.is_empty() {
                    (tr("settings-keybind-unbound").to_string(), 0.4)
                } else {
                    (crate::domain::input::binding_label(shown), 1.0)
                };
                let slot_focused = keyboard_focused && focused_choice == Some(slot);
                slots = slots.push(
                    button(
                        text(display)
                            .font(crate::frontend::ui::TEXT_FONT)
                            .size(13.0 * legible_type_scale(scale))
                            .color(with_alpha(palette.surface_text, alpha * fade)),
                    )
                    .padding([8.0 * scale, 10.0 * scale])
                    .width(Length::Fill)
                    .on_press(Message::Settings(SettingsMsg::KeybindCapture(path.clone(), slot)))
                    .style(move |_theme, status| {
                        crate::frontend::ui::tile_action_style(
                            &pal,
                            fade,
                            false,
                            slot_focused,
                            status,
                        )
                    }),
                );
            }
            slots.into()
        }
        Control::TextField { key, placeholder, .. } => {
            let value = values.get(&key).map_or("", String::as_str);
            let enabled = key != PRESET_NAME_KEY || has_selected_preset;
            let hint =
                if enabled { placeholder } else { tr("settings-selector-preset-rename-hint") };
            let mut input = text_input(hint, value)
                .id(super::super::workbench_input_id(&key))
                .font(crate::frontend::ui::TEXT_FONT)
                .size(13.0 * legible_type_scale(scale))
                .padding([8.0 * scale, 10.0 * scale])
                .width(Length::Fill)
                .style(move |_theme, status| {
                    crate::frontend::ui::quiet_input_style(palette, fade, status)
                });
            if enabled {
                if key == PRESET_NAME_KEY {
                    input = input.on_input(Message::PresetNameInput);
                } else {
                    let message_key = key.clone();
                    input = input
                        .on_input(move |raw| {
                            Message::Settings(SettingsMsg::Input(message_key.clone(), raw))
                        })
                        .on_submit(Message::Settings(SettingsMsg::Commit));
                }
            }
            input.into()
        }
        Control::Resolution { key, width_placeholder, height_placeholder, .. } => {
            let value = values.get(&key).map_or("", String::as_str);
            let maximum = key.ends_with(".to");
            if maximum && value.is_empty() {
                return row![
                    label(
                        tr("settings-filter-resolution-unlimited"),
                        13.0,
                        scale,
                        with_alpha(palette.surface_text, fade)
                    ),
                    option_button(
                        tr("settings-filter-resolution-set-limit").into(),
                        false,
                        false,
                        Message::Settings(SettingsMsg::ResolutionLimit(key, true)),
                        keyboard_focused,
                        scale,
                        palette,
                        fade,
                        motion,
                    ),
                ]
                .spacing(12.0 * scale)
                .align_y(Alignment::Center)
                .into();
            }
            let (width, height) = value.split_once(['x', 'X', '×']).unwrap_or(("", ""));
            let width_key = key.clone();
            let height_key = key.clone();
            let width_input = text_input(width_placeholder, width)
                .id(super::super::workbench_input_id(&format!("{key}.width")))
                .font(crate::frontend::ui::TEXT_FONT)
                .on_input(move |raw| {
                    Message::Settings(SettingsMsg::ResolutionInput(width_key.clone(), true, raw))
                })
                .on_submit(Message::Settings(SettingsMsg::Commit))
                .size(13.0 * legible_type_scale(scale))
                .padding([8.0 * scale, 10.0 * scale])
                .width(Length::Fill)
                .style(move |_theme, status| {
                    crate::frontend::ui::quiet_input_style(palette, fade, status)
                });
            let height_input = text_input(height_placeholder, height)
                .id(super::super::workbench_input_id(&format!("{key}.height")))
                .font(crate::frontend::ui::TEXT_FONT)
                .on_input(move |raw| {
                    Message::Settings(SettingsMsg::ResolutionInput(height_key.clone(), false, raw))
                })
                .on_submit(Message::Settings(SettingsMsg::Commit))
                .size(13.0 * legible_type_scale(scale))
                .padding([8.0 * scale, 10.0 * scale])
                .width(Length::Fill)
                .style(move |_theme, status| {
                    crate::frontend::ui::quiet_input_style(palette, fade, status)
                });
            let dimensions = row![
                column![
                    label(
                        width_placeholder,
                        11.0,
                        scale,
                        with_alpha(palette.surface_text, 0.68 * fade)
                    ),
                    width_input
                ]
                .spacing(4.0 * scale)
                .width(Length::Fill),
                label("×", 14.0, scale, with_alpha(palette.surface_text, 0.56 * fade)),
                column![
                    label(
                        height_placeholder,
                        11.0,
                        scale,
                        with_alpha(palette.surface_text, 0.68 * fade)
                    ),
                    height_input
                ]
                .spacing(4.0 * scale)
                .width(Length::Fill),
            ]
            .spacing(8.0 * scale)
            .align_y(Alignment::End);
            let mut editor = column![dimensions].spacing(8.0 * scale);
            if maximum {
                editor = editor.push(option_button(
                    tr("settings-filter-resolution-clear-limit").into(),
                    false,
                    false,
                    Message::Settings(SettingsMsg::ResolutionLimit(key.clone(), false)),
                    false,
                    scale,
                    palette,
                    fade,
                    motion,
                ));
            }
            let base = key.rsplit_once('.').map_or("", |(base, _)| base);
            let from = values.get(&format!("{base}.from")).map_or("", String::as_str);
            let to = values.get(&format!("{base}.to")).map_or("", String::as_str);
            if !crate::domain::library::filter::resolution_bounds_valid(from, to) {
                editor = editor.push(label(
                    tr("settings-filter-resolution-invalid"),
                    11.0,
                    scale,
                    with_alpha(palette.primary, fade),
                ));
            }
            editor.into()
        }
        Control::Dropdown { path, options, current, palettes } => choice_buttons(
            &path,
            options,
            &current,
            &[],
            &palettes,
            keyboard_focused,
            focused_choice,
            available_width,
            scale,
            palette,
            fade,
            motion,
        ),
        Control::Chips { path, options, current, disabled, background } => {
            let choices = choice_buttons(
                &path,
                options,
                &current,
                &disabled,
                &[],
                keyboard_focused,
                focused_choice,
                available_width,
                scale,
                palette,
                fade,
                motion,
            );
            if let Some(background) = background {
                let shown = matches!(current.as_str(), "center" | "fit");
                let reveal = reveals
                    .and_then(|maps| maps.bars.get(&format!("background:{path}")))
                    .map_or(if shown { 1.0 } else { 0.0 }, |value| value.x);
                let picker = reveals
                    .and_then(|maps| maps.bars.get(&background.picker_id()))
                    .map_or(0.0, |value| value.x);
                column![choices, background.view(reveal, picker, scale, palette)].into()
            } else {
                choices
            }
        }
        Control::MotionWeights { weights } => {
            let mut group = row![].spacing(8.0 * scale);
            for (label, key, reset) in weights {
                let value = values.get(&key).map_or("", String::as_str);
                let message_key = key.clone();
                group = group.push(
                    column![
                        self::label(
                            label,
                            10.0,
                            scale,
                            with_alpha(palette.surface_text, 0.62 * fade),
                        ),
                        row![
                            text_input("0", value)
                                .id(super::super::workbench_input_id(&key))
                                .font(crate::frontend::ui::TEXT_FONT)
                                .on_input(move |raw| Message::Settings(SettingsMsg::Input(
                                    message_key.clone(),
                                    raw,
                                )))
                                .on_submit(Message::Settings(SettingsMsg::Commit))
                                .size(13.0 * legible_type_scale(scale))
                                .padding([6.0 * scale, 8.0 * scale])
                                .style(move |_theme, status| {
                                    crate::frontend::ui::quiet_input_style(palette, fade, status)
                                }),
                            fixed_button(
                                "↺".into(),
                                false,
                                false,
                                Some(Message::Settings(SettingsMsg::Run(reset))),
                                false,
                                scale * 0.85,
                                palette,
                                fade,
                                34.0 * scale,
                                motion,
                            ),
                        ]
                        .spacing(5.0 * scale),
                    ]
                    .spacing(5.0 * scale)
                    .width(Length::FillPortion(1)),
                );
            }
            group.into()
        }
        Control::ActionBtn { id, label } => {
            let confirming = armed == Some(&id);
            let confirmation = crate::i18n::tr_args!("settings-control-confirm", label => &label);
            let width = button_width(&confirmation, scale).max(button_width(&label, scale));
            action_button(
                if confirming { confirmation } else { label },
                confirming,
                Message::Settings(SettingsMsg::Run(id)),
                keyboard_focused,
                scale,
                palette,
                fade,
                width,
            )
        }
        Control::ActionChips { items } | Control::Tile { items, .. } => wrapped_options(
            items
                .into_iter()
                .enumerate()
                .map(|(index, (id, label))| {
                    let width = button_width(&label, scale);
                    (
                        width,
                        action_button(
                            label,
                            false,
                            Message::Settings(SettingsMsg::Run(id)),
                            keyboard_focused && focused_choice == Some(index),
                            scale,
                            palette,
                            fade,
                            width,
                        ),
                    )
                })
                .collect(),
            available_width,
            scale,
        ),
        Control::Presets { mode, items } => {
            let mut options = vec![(
                button_width(tr("settings-selector-preset-save-current"), scale),
                option_button(
                    tr("settings-selector-preset-save-current").into(),
                    false,
                    false,
                    Message::SavePreset,
                    keyboard_focused && focused_choice == Some(0),
                    scale,
                    palette,
                    fade,
                    motion,
                ),
            )];
            for (index, (name, active)) in items.into_iter().enumerate() {
                let apply_index = 1 + index * 2;
                let delete_index = apply_index + 1;
                let width = button_width(&name, scale);
                options.push((
                    width,
                    fixed_button(
                        name.clone(),
                        active,
                        false,
                        Some(Message::ApplyPreset(mode.clone(), name.clone())),
                        keyboard_focused && focused_choice == Some(apply_index),
                        scale,
                        palette,
                        fade,
                        width,
                        motion,
                    ),
                ));
                options.push((
                    34.0 * scale,
                    fixed_button(
                        "×".into(),
                        false,
                        true,
                        Some(Message::DeletePreset(mode.clone(), name)),
                        keyboard_focused && focused_choice == Some(delete_index),
                        scale,
                        palette,
                        fade,
                        34.0 * scale,
                        motion,
                    ),
                ));
            }
            wrapped_options(options, available_width, scale)
        }
        Control::Details { summary, .. } | Control::StackBar { summary, .. } => label(
            format!("▸ {summary}"),
            11.0,
            scale,
            with_alpha(palette.surface_text, 0.68 * fade),
        )
        .into(),
        Control::Segment | Control::Static => text("").into(),
        Control::Code { snippet } => label(
            snippet,
            crate::frontend::ui::TYPE_SMALL,
            scale,
            with_alpha(palette.surface_text, 0.68 * fade),
        )
        .into(),
        Control::Preview => label(
            tr("settings-control-live-backdrop"),
            12.0,
            scale,
            with_alpha(palette.primary, fade),
        )
        .into(),
    }
}
