#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use iced::widget::canvas::{self, Frame, Path, Stroke};
use iced::widget::{column, container, image, mouse_area, scrollable, stack, text, text_input};
use iced::{Alignment, Background, Color, Element, Length, Padding, Point, Rectangle};

use crate::app::Message;
use crate::contracts::settings::SettingsSource;
use crate::frontend::animation::{MotionProfile, Tween};
use crate::frontend::components::with_alpha;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{
    FOLIO_RULE_ALPHA, TYPE_SMALL, end, folio_horizontal_rule, label, legible_type_scale, mirror_x,
    row,
};
use crate::i18n::tr;

use super::super::{
    ActionId, Card, Control, Row, SettingsFocus, SettingsMsg, build_tab_with_runtime_status,
};
use super::control;

const GLYPH_FADE: f32 = 1.0;

pub struct SourceCtx<'a> {
    pub config: &'a dyn SettingsSource,
    pub values: &'a HashMap<String, String>,
    pub themes: &'a [String],
    pub folders: &'a [String],
    pub backends: &'a [String],
    pub outputs: &'a [String],
    pub output_statuses: &'a [crate::contracts::daemon::OutputStatus],
    pub output_previews: &'a HashMap<String, String>,
    pub library_watch: Option<&'a crate::contracts::daemon::LibraryWatchStatus>,
    pub app_themes: Option<&'a crate::contracts::daemon::AppThemesResult>,
    pub playback: Option<&'a crate::contracts::daemon::PlaybackStatus>,
    pub weather: Option<&'a crate::contracts::daemon::WeatherStatus>,
    pub weather_pending: bool,
    pub playlists: &'a [crate::contracts::playlists::Playlist],
    pub sddm_themes: &'a [String],
    pub thumbnail_task: Option<&'a crate::contracts::daemon::TaskStatus>,
    pub analysis: String,
}

impl SourceCtx<'_> {
    pub(super) fn motion(&self) -> MotionProfile {
        MotionProfile::new(
            self.config.motion_fast_ms(),
            self.config.motion_standard_ms(),
            self.config.motion_slow_ms(),
        )
    }

    pub(super) fn cards(&self, tab: &str) -> Vec<(Card, Vec<Row>)> {
        let mut cards = build_tab_with_runtime_status(
            tab,
            self.config,
            self.themes,
            self.folders,
            &self.analysis,
            self.backends,
            super::super::RuntimeStatus {
                outputs: self.outputs,
                statuses: self.output_statuses,
                previews: Some(self.output_previews),
                library_watch: self.library_watch,
                playback: self.playback,
                app_themes: self.app_themes,
                weather: self.weather,
                weather_pending: self.weather_pending,
                playlists: self.playlists,
                sddm_themes: self.sddm_themes,
            },
        );
        if let Some(task) = self.thumbnail_task {
            for row in cards.iter_mut().flat_map(|(_, rows)| rows) {
                if let super::super::Control::ActionBtn {
                    id: ActionId::CaptureWeThumbnails,
                    label,
                } = &mut row.control
                {
                    let state = match &task.state {
                        crate::contracts::daemon::TaskState::Running => {
                            tr("filter-bar-state-running")
                        }
                        crate::contracts::daemon::TaskState::Paused => {
                            tr("filter-bar-state-paused")
                        }
                        crate::contracts::daemon::TaskState::Completed => {
                            tr("filter-bar-state-done")
                        }
                        crate::contracts::daemon::TaskState::Failed => {
                            tr("filter-bar-state-failed")
                        }
                        crate::contracts::daemon::TaskState::Cancelled => {
                            tr("filter-bar-state-stopped")
                        }
                        crate::contracts::daemon::TaskState::Other(state) => state.as_str(),
                    };
                    row.desc =
                        format!("{state} · {} / {} · {}", task.progress, task.total, task.detail);
                    if task.state.is_active() {
                        *label = tr("settings-performance-capture-we-stop").into();
                    }
                }
            }
        }
        cards
    }
}

#[derive(Clone, Copy)]
pub struct FocusCtx<'a> {
    pub keyboard_focus: SettingsFocus,
    pub focused_control: usize,
    pub focused_choice: Option<usize>,
    pub expanded_details: &'a HashSet<String>,
    pub bar_reveals: &'a HashMap<String, Tween>,
    pub active_input: Option<&'a str>,
    pub armed: Option<&'a ActionId>,
}

impl<'a> FocusCtx<'a> {
    fn on(&self, area: SettingsFocus, index: usize) -> (bool, Option<usize>) {
        let focused = self.keyboard_focus == area && self.focused_control == index;
        (focused, if focused { self.focused_choice } else { None })
    }

    pub(super) fn reveals(&self) -> control::Reveals<'a> {
        control::Reveals { bars: self.bar_reveals, folds: self.expanded_details }
    }

    fn bar(&self, id: &str) -> (bool, f32) {
        self.bar_reveals.get(id).map_or((false, 0.0), |reveal| (reveal.target > 0.5, reveal.x))
    }
}

#[derive(Clone, Copy)]
pub struct ChromeCtx<'a> {
    pub viewport: (f32, f32),
    /// Settings → Picker → Settings window (size, place and scale).
    pub window: crate::frontend::ui::SheetWindow,
    /// Reading direction, for the side of the layout studio.
    pub rtl: bool,
    pub entrance: f32,
    pub palette: &'a Palette,
}

pub struct KeybindCaptureView {
    pub title: String,
    pub binding: String,
    pub edited: bool,
    pub conflict: Option<String>,
}

pub struct ProcessPickerView {
    pub targets: Vec<String>,
    pub running: Vec<String>,
    pub query: String,
    pub manual: String,
    pub scroll: f32,
}

#[derive(Clone, Copy)]
pub struct ProfilesCtx<'a> {
    pub selected: usize,
    pub filter: &'a str,
    pub enter: f32,
    pub marker: f32,
}

#[derive(Clone, Copy)]
pub struct FoldersCtx<'a> {
    pub drafts: &'a HashMap<&'static str, String>,
    pub undo: Option<(crate::contracts::settings::FolderKind, &'a str)>,
}

enum Page<'a> {
    Profiles(super::profiles_page::ProfilesView<'a>),
    Folders(super::folders_page::FoldersView<'a>),
}

fn page_view<'a>(
    tab: &str,
    section: usize,
    source: &SourceCtx<'a>,
    focus: FocusCtx<'a>,
    profiles: ProfilesCtx<'a>,
    folders: FoldersCtx<'a>,
) -> Option<Page<'a>> {
    use super::super::PageSection;
    match super::super::page_section(tab, section)? {
        PageSection::Profiles => Some(Page::Profiles(super::profiles_page::ProfilesView {
            page: super::super::profiles_page(
                source.config,
                source.folders,
                profiles.selected,
                profiles.filter,
            ),
            filter: profiles.filter,
            enter: profiles.enter,
            marker: profiles.marker,
            values: source.values,
            reveals: focus.reveals(),
            armed: focus.armed,
        })),
        PageSection::Folders(kind) => Some(Page::Folders(super::folders_page::FoldersView {
            ledger: super::super::folder_ledger(source.config, kind, source.library_watch),
            draft: folders.drafts.get(kind.key()).map_or("", String::as_str),
            undo: folders.undo.filter(|(undone, _)| *undone == kind).map(|(_, path)| path),
            values: source.values,
            editing: focus.active_input,
            bars: focus.bar_reveals,
        })),
    }
}

pub struct WorkbenchInput<'a> {
    pub source: SourceCtx<'a>,
    pub focus: FocusCtx<'a>,
    pub chrome: ChromeCtx<'a>,
    pub profiles: ProfilesCtx<'a>,
    pub folders: FoldersCtx<'a>,
    /// R1: pages, menus, Home, palette and toast.
    pub nav: super::shell::NavView<'a>,
    pub tab: &'a str,
    pub selected_section: usize,
    pub keybind_capture: Option<KeybindCaptureView>,
    pub process_picker: Option<ProcessPickerView>,
    pub weather_review: Option<super::weather_review::WeatherReviewView>,
}

/// Title of the heading row (`Control::Segment`) above row `focused`.
fn segment_of(rows: &[Row], focused: usize) -> Option<String> {
    rows.iter()
        .take(focused.saturating_add(1).min(rows.len()))
        .rev()
        .find(|row| matches!(row.control, Control::Segment))
        .map(|row| row.title.clone())
}

#[allow(clippy::too_many_lines)]
pub fn settings_workbench<'a>(input: WorkbenchInput<'a>) -> Element<'a, Message> {
    let WorkbenchInput {
        source,
        focus,
        chrome,
        profiles,
        folders,
        nav,
        tab,
        selected_section,
        keybind_capture,
        process_picker,
        weather_review,
    } = input;
    let ChromeCtx { viewport, window: sheet, entrance, palette, .. } = chrome;
    let scale = sheet.scale();
    let motion = source.motion();
    let reveal = crate::frontend::animation::smoothstep(entrance.clamp(0.0, 1.0));
    let (panel_width, panel_height) = sheet.dims(viewport);
    let bar = super::shell::top_bar(
        &nav,
        panel_width,
        scale,
        palette,
        reveal,
        super::shell::bar_has_focus(focus.keyboard_focus),
    );
    // Pages of rows keep a reading width; pages with cards (and Home) use
    // the window.
    let wide_width = super::cards::wide_page_width(panel_width, scale);
    let content_width = wide_width.min(820.0 * scale);
    let body: Element<'a, Message> = match nav.page {
        None => super::home::home(
            &nav,
            focus.keyboard_focus,
            wide_width,
            panel_height - super::shell::bar_height(scale),
            scale,
            palette,
            reveal,
        ),
        Some(page) => {
            if let Some(view) = page_view(tab, selected_section, &source, focus, profiles, folders)
            {
                let paint = super::page::Paint { scale, palette, fade: reveal, motion };
                let content = match view {
                    Page::Profiles(view) => super::profiles_page::profiles_page(view, paint),
                    Page::Folders(view) => super::folders_page::folders_page(view, paint),
                };
                super::shell::page_frame(
                    &nav,
                    page,
                    None,
                    0,
                    content,
                    content_width,
                    scale,
                    palette,
                    reveal,
                )
            } else {
                let mut build = |name: &str| source.cards(name);
                let full =
                    super::super::nav::page_content(&super::super::nav::PAGES[page], &mut build)
                        .rows(false);
                let hidden = if nav.hides_advanced {
                    full.iter().filter(|row| super::super::nav::row_is_advanced(row)).count()
                } else {
                    0
                };
                let rows =
                    if nav.hides_advanced { super::super::nav::simple_rows(full) } else { full };
                let has_selected_preset = rows.iter().any(|setting| {
                    matches!(&setting.control, Control::Presets { items, .. } if items.iter().any(|(_, active)| *active))
                });
                let segment = if focus.keyboard_focus == SettingsFocus::Controls {
                    segment_of(&rows, focus.focused_control)
                } else {
                    None
                };
                let content_width = if rows.iter().any(|row| {
                    matches!(row.control, Control::Tile { .. })
                        || super::super::nav::gallery(&row.control).is_some()
                }) {
                    wide_width
                } else {
                    content_width
                };
                let content = super::rows::rows(
                    super::rows::RowsInput {
                        pictures: nav.pictures,
                        playing: nav.playing,
                        tile_hover: nav.tile_hover,
                        tile_fade: nav.tile_fade,
                        follow_cursor: nav.follow_cursor,
                        cursor: nav.cursor,
                        pending: nav.strips_pending,
                        card_layout: nav.card_layout,
                        rows,
                        values: source.values,
                        has_selected_preset,
                        width: content_width,
                        scale,
                        palette,
                        fade: reveal,
                        motion,
                        cfg: source.config,
                        more: nav.more,
                    },
                    focus,
                );
                super::shell::page_frame(
                    &nav,
                    page,
                    segment,
                    hidden,
                    content,
                    content_width,
                    scale,
                    palette,
                    reveal,
                )
            }
        }
    };
    let on_home = nav.page.is_none();
    let picture = nav.backdrop.picture.clone().filter(|_| on_home);
    // Home: the wallpaper under a dark veil. Pages: the bare sheet with an
    // optional faint grid.
    let backdrop: Element<'a, Message> = match picture {
        Some(handle) => stack![
            super::shell::backdrop_image(handle, reveal),
            container(text("")).width(Length::Fill).height(Length::Fill).style(move |_| {
                crate::frontend::ui::veil_style(0.8 * reveal, 0.46 * reveal, 0.9 * reveal)
            }),
        ]
        .into(),
        None if nav.grid => {
            crate::frontend::ui::grid_pattern(palette, 0.034 * reveal, 40.0 * scale)
        }
        None => iced::widget::Space::new().width(Length::Fill).height(Length::Fill).into(),
    };
    let blurred_bar: Element<'a, Message> = match (nav.bar_blur, nav.backdrop.blurred.clone()) {
        (true, Some(handle)) => container(stack![
            iced::widget::image(handle)
                .width(Length::Fill)
                .height(Length::Fixed(panel_height))
                .content_fit(iced::ContentFit::Cover)
                .opacity(reveal),
            container(text(""))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| { crate::frontend::ui::ink_fill(palette.surface, 0.55 * reveal) }),
        ])
        .width(Length::Fill)
        .height(Length::Fixed(super::shell::bar_height(scale)))
        .clip(true)
        .into(),
        _ => iced::widget::Space::new().width(Length::Fill).height(Length::Fixed(0.0)).into(),
    };
    let bar_rule = super::shell::rule(palette, if on_home { 0.0 } else { 0.14 * reveal });
    let layout =
        column![stack![blurred_bar, bar], bar_rule, body].width(Length::Fill).height(Length::Fill);
    let mut window_layers = stack![backdrop, layout];
    if let Some(big) = &nav.big {
        window_layers = window_layers.push(super::shell::big_preview_layer(
            big,
            nav.backdrop.blurred.clone(),
            (panel_width, panel_height),
            scale,
            palette,
            reveal,
        ));
    }
    if let Some((text_, can_undo)) = nav.toast.clone() {
        window_layers = window_layers.push(super::shell::toast_layer(
            text_,
            can_undo,
            scale,
            palette,
            nav.toast_fade * reveal,
        ));
    }
    if let Some(view) = &nav.palette {
        window_layers =
            window_layers.push(super::shell::palette_layer(view, scale, palette, reveal));
    }
    let panel = container(window_layers)
        .width(Length::Fixed(panel_width))
        .height(Length::Fixed(panel_height))
        .clip(true)
        .style(move |_| crate::frontend::ui::sheet_style(palette, reveal));
    let mut layers = stack![
        mouse_area(
            container(text(""))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| crate::frontend::ui::folio_scrim_style(reveal)),
        )
        .on_press(Message::ToggleSettings),
        sheet.place(iced::widget::opaque(panel)),
    ];
    if let Some(capture) = keybind_capture {
        layers = layers.push(keybind_capture_layer(capture, scale, palette, reveal));
    }
    if let Some(picker) = process_picker {
        layers = layers.push(process_picker_layer(picker, scale, palette, reveal));
    }
    if let Some(review) = weather_review {
        layers = layers.push(super::weather_review::layer(review, scale, palette, reveal));
    }
    layers.into()
}

fn process_picker_layer(
    picker: ProcessPickerView,
    scale: f32,
    palette: &Palette,
    reveal: f32,
) -> Element<'_, Message> {
    let ProcessPickerView { targets, running, query, manual, scroll } = picker;
    let rail_count = running.len().max(2);
    let rail_active = (scroll.clamp(0.0, 1.0) * (rail_count - 1) as f32).round() as usize;
    let target_rows: Element<'_, Message> = if targets.is_empty() {
        container(label(
            tr("settings-playback-targets-empty"),
            TYPE_SMALL,
            scale,
            with_alpha(palette.surface_text, 0.58),
        ))
        .width(Length::Fill)
        .padding([12.0 * scale, 14.0 * scale])
        .style(move |_| {
            crate::frontend::ui::box_style(
                with_alpha(palette.surface_container, 0.46),
                with_alpha(palette.outline, 0.3),
            )
        })
        .into()
    } else {
        targets
            .into_iter()
            .fold(column![].spacing(6.0 * scale), |rows, target| {
                let remove = crate::frontend::ui::folio_destructive_action(
                    tr("settings-playback-target-remove"),
                    false,
                    Some(Message::Settings(SettingsMsg::ProcessPickerRemove(target.clone()))),
                    Length::Fixed(80.0 * scale),
                    scale * 0.76,
                    palette,
                );
                rows.push(
                    container(
                        row![
                            label(
                                target,
                                14.0,
                                scale,
                                with_alpha(palette.surface_text, GLYPH_FADE)
                            ),
                            container(text("")).width(Length::Fill),
                            remove,
                        ]
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .padding([8.0 * scale, 10.0 * scale])
                    .style(move |_| {
                        crate::frontend::ui::box_style(
                            with_alpha(palette.surface_container, 0.7),
                            with_alpha(palette.primary, 0.46),
                        )
                    }),
                )
            })
            .into()
    };
    let running_rows: Element<'_, Message> = if running.is_empty() {
        container(label(
            tr("settings-playback-running-empty"),
            TYPE_SMALL,
            scale,
            with_alpha(palette.surface_text, 0.58),
        ))
        .width(Length::Fill)
        .padding([12.0 * scale, 14.0 * scale])
        .style(move |_| {
            crate::frontend::ui::box_style(
                with_alpha(palette.surface_container, 0.46),
                with_alpha(palette.outline, 0.3),
            )
        })
        .into()
    } else {
        running
            .into_iter()
            .fold(column![].spacing(6.0 * scale), |rows, process| {
                let add = crate::frontend::ui::folio_action(
                    tr("settings-playback-target-add"),
                    true,
                    Some(Message::Settings(SettingsMsg::ProcessPickerAdd(process.clone()))),
                    Length::Fixed(68.0 * scale),
                    scale * 0.76,
                    palette,
                );
                rows.push(
                    container(
                        row![
                            label(
                                process,
                                14.0,
                                scale,
                                with_alpha(palette.surface_text, GLYPH_FADE)
                            ),
                            container(text("")).width(Length::Fill),
                            add,
                        ]
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .padding([8.0 * scale, 10.0 * scale])
                    .style(move |_| {
                        crate::frontend::ui::box_style(
                            with_alpha(palette.surface_container, 0.7),
                            with_alpha(palette.outline, 0.38),
                        )
                    }),
                )
            })
            .into()
    };
    let close = crate::frontend::ui::folio_action(
        tr("settings-playback-target-close"),
        true,
        Some(Message::Settings(SettingsMsg::ProcessPickerClose)),
        Length::Fixed(76.0 * scale),
        scale * 0.82,
        palette,
    );
    let body = column![
        row![
            label(
                tr("settings-playback-target-dialog-title"),
                20.0,
                scale,
                with_alpha(palette.surface_text, GLYPH_FADE),
            ),
            container(text("")).width(Length::Fill),
            label(
                tr("settings-playback-process-enabled"),
                TYPE_SMALL,
                scale,
                with_alpha(palette.primary, GLYPH_FADE),
            ),
        ]
        .align_y(Alignment::Center),
        label(
            tr("settings-playback-target-dialog-desc"),
            TYPE_SMALL,
            scale,
            with_alpha(palette.surface_text, 0.65),
        )
        .line_height(iced::widget::text::LineHeight::Relative(1.35)),
        crate::frontend::ui::folio_horizontal_rule(with_alpha(palette.outline, 0.46)),
        label(
            tr("settings-playback-targets"),
            12.0,
            scale,
            with_alpha(palette.primary, GLYPH_FADE)
        ),
        target_rows,
        label(
            tr("settings-playback-running"),
            12.0,
            scale,
            with_alpha(palette.primary, GLYPH_FADE)
        ),
        row![
            label("⌕", 15.0, scale, with_alpha(palette.primary, GLYPH_FADE)),
            text_input(tr("browser-search-placeholder"), &query)
                .font(crate::frontend::ui::UI_FONT)
                .on_input(|query| Message::Settings(SettingsMsg::ProcessPickerSearch(query)))
                .size(12.0 * crate::frontend::ui::legible_type_scale(scale))
                .padding([8.0 * scale, 9.0 * scale])
                .width(Length::Fill)
                .style(move |_theme, _status| {
                    crate::frontend::ui::workbench_input_style(
                        palette.surface_text,
                        palette.primary,
                    )
                }),
        ]
        .spacing(8.0 * scale)
        .align_y(Alignment::Center),
        row![
            text_input(tr("settings-playback-target-manual-placeholder"), &manual)
                .font(crate::frontend::ui::UI_FONT)
                .on_input(|process| Message::Settings(SettingsMsg::ProcessPickerManualInput(
                    process
                )))
                .on_submit(Message::Settings(SettingsMsg::ProcessPickerManualAdd))
                .size(12.0 * crate::frontend::ui::legible_type_scale(scale))
                .padding([8.0 * scale, 9.0 * scale])
                .width(Length::Fill)
                .style(move |_theme, _status| {
                    crate::frontend::ui::workbench_input_style(
                        palette.surface_text,
                        palette.primary,
                    )
                }),
            crate::frontend::ui::folio_action(
                tr("settings-playback-target-add"),
                true,
                Some(Message::Settings(SettingsMsg::ProcessPickerManualAdd)),
                Length::Fixed(68.0 * scale),
                scale * 0.76,
                palette,
            ),
        ]
        .spacing(8.0 * scale)
        .align_y(Alignment::Center),
        row![
            scrollable(running_rows)
                .height(Length::Fixed(142.0 * scale))
                .width(Length::Fill)
                .direction(iced::widget::scrollable::Direction::Vertical(
                    iced::widget::scrollable::Scrollbar::new().width(0.0).scroller_width(0.0),
                ))
                .on_scroll(|viewport| {
                    Message::Settings(SettingsMsg::ProcessPickerScroll(
                        viewport.relative_offset().y,
                    ))
                }),
            iced::widget::canvas(IndexRail {
                active: rail_active,
                count: rail_count,
                notches: false,
                palette: *palette,
                fade: GLYPH_FADE,
            })
            .width(Length::Fixed(16.0 * scale))
            .height(Length::Fixed(142.0 * scale)),
        ]
        .spacing(8.0 * scale),
        crate::frontend::ui::folio_horizontal_rule(with_alpha(palette.outline, 0.46)),
        row![container(text("")).width(Length::Fill), close].align_y(Alignment::Center),
    ]
    .spacing(12.0 * scale);
    let panel = mouse_area(
        container(body)
            .width(Length::Fixed(500.0 * scale.max(0.9)))
            .padding([20.0 * scale, 22.0 * scale])
            .style(move |_| crate::frontend::ui::folio_sheet_panel_style(palette, reveal)),
    )
    .on_press(Message::Noop);
    mouse_area(
        container(panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_| crate::frontend::ui::folio_scrim_style(0.6 * reveal)),
    )
    .on_press(Message::Settings(SettingsMsg::ProcessPickerClose))
    .into()
}

fn keybind_capture_layer(
    capture: KeybindCaptureView,
    scale: f32,
    palette: &Palette,
    reveal: f32,
) -> Element<'_, Message> {
    let KeybindCaptureView { title, binding, edited, conflict } = capture;
    let action_scale = scale * 0.82;
    let action = |label_key: &'static str, active: bool, message: Message| {
        crate::frontend::ui::folio_action(
            tr(label_key),
            active,
            Some(message),
            Length::Fixed(crate::frontend::ui::folio_action_width(tr(label_key), action_scale)),
            action_scale,
            palette,
        )
    };
    let pad = mouse_area(
        container(label(
            binding,
            22.0,
            scale,
            with_alpha(if edited { palette.primary } else { palette.surface_text }, GLYPH_FADE),
        ))
        .width(Length::Fill)
        .center_x(Length::Fill)
        .padding([20.0 * scale, 12.0 * scale])
        .style(move |_| {
            crate::frontend::ui::box_style(
                with_alpha(palette.surface_container, 0.55),
                with_alpha(if edited { palette.primary } else { palette.outline }, 0.75),
            )
        }),
    )
    .on_press(Message::Settings(SettingsMsg::KeybindCaptureClick(
        crate::domain::input::MouseButton::Left,
    )))
    .on_right_press(Message::Settings(SettingsMsg::KeybindCaptureClick(
        crate::domain::input::MouseButton::Right,
    )))
    .on_middle_press(Message::Settings(SettingsMsg::KeybindCaptureClick(
        crate::domain::input::MouseButton::Middle,
    )));
    let mut body = column![
        label(title, 14.0, scale, with_alpha(palette.surface_text, GLYPH_FADE)),
        pad,
        label(
            tr("settings-keybind-capture-hint"),
            TYPE_SMALL,
            scale,
            with_alpha(palette.surface_text, 0.58),
        )
        .line_height(iced::widget::text::LineHeight::Relative(1.35)),
    ]
    .spacing(10.0 * scale);
    if let Some(conflict) = conflict {
        body =
            body.push(label(conflict, TYPE_SMALL, scale, with_alpha(palette.tertiary, GLYPH_FADE)));
    }
    body = body.push(
        row![
            action("settings-keybind-capture-default", false, {
                Message::Settings(SettingsMsg::KeybindCaptureDefault)
            }),
            action("settings-keybind-capture-unbind", false, {
                Message::Settings(SettingsMsg::KeybindCaptureUnbind)
            }),
            container(text("")).width(Length::Fill),
            action("settings-keybind-capture-cancel", false, {
                Message::Settings(SettingsMsg::KeybindCaptureCancel)
            }),
            action("settings-keybind-capture-ok", true, {
                Message::Settings(SettingsMsg::KeybindCaptureApply)
            }),
        ]
        .spacing(8.0 * scale)
        .align_y(Alignment::Center),
    );
    let panel = mouse_area(
        container(body)
            .width(Length::Fixed(390.0 * scale.max(0.9)))
            .padding([18.0 * scale, 20.0 * scale])
            .style(move |_| crate::frontend::ui::folio_sheet_panel_style(palette, reveal)),
    )
    .on_press(Message::Noop);
    mouse_area(
        container(panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(move |_| crate::frontend::ui::folio_scrim_style(0.6 * reveal)),
    )
    .on_press(Message::Settings(SettingsMsg::KeybindCaptureCancel))
    .into()
}

pub fn picker_layout_workbench<'a>(
    source: &SourceCtx<'a>,
    focus: FocusCtx<'a>,
    chrome: ChromeCtx<'a>,
) -> Element<'a, Message> {
    const LAYOUT_SECTION: usize = 1;
    let ChromeCtx { viewport, window, rtl, entrance, palette } = chrome;
    let scale = window.scale();
    let motion = source.motion();
    let rows = source
        .cards("picker")
        .into_iter()
        .nth(LAYOUT_SECTION)
        .map_or_else(Vec::new, |(_, rows)| rows);
    let has_selected_preset = rows.iter().any(|row| {
        matches!(&row.control, Control::Presets { items, .. } if items.iter().any(|(_, active)| *active))
    });
    let reveal = crate::frontend::animation::smoothstep(entrance.clamp(0.0, 1.0));
    let panel_width = super::super::picker_layout_studio_width(viewport.0, &window);
    let panel_height = super::super::picker_layout_studio_height(viewport.1, &window);

    let mut pinned = column![].spacing(10.0 * scale);
    let mut controls = column![].spacing(13.0 * scale);
    for (index, setting) in rows.into_iter().enumerate() {
        let is_pinned = setting.title == tr("settings-selector-display-mode-label")
            || matches!(
                &setting.control,
                Control::TextField { key, .. } if key == super::super::PRESET_NAME_KEY
            )
            || matches!(&setting.control, Control::Presets { .. });
        let field = picker_layout_field(
            setting,
            index,
            source.values,
            has_selected_preset,
            focus,
            panel_width - 42.0 * scale,
            scale,
            palette,
            GLYPH_FADE,
            motion,
        );
        if is_pinned {
            pinned = pinned.push(field);
        } else {
            controls = controls.push(field);
        }
    }

    let mode = tr(match source.config.display_mode().as_str() {
        "grid" | "wall" => "settings-selector-mode-wall",
        "hex" => "settings-selector-mode-hex",
        "sandy" | "nova" => "settings-selector-mode-sandy",
        "hand" => "settings-selector-mode-hand",
        "depth" => "settings-selector-mode-depth",
        "collection" => "settings-selector-mode-collection",
        "coverflow" => "settings-selector-mode-coverflow",
        "showcase" => "settings-selector-mode-showcase",
        _ => "settings-selector-mode-slices",
    });
    let page_actions = row![
        crate::frontend::ui::folio_action(
            tr("settings-studio-back"),
            false,
            Some(Message::Settings(SettingsMsg::LeaveLayoutStudio)),
            Length::Fixed(104.0 * scale.max(1.0)),
            scale,
            palette,
        ),
        container(text("")).width(Length::Fill),
        crate::frontend::ui::folio_action(
            "×",
            false,
            Some(Message::ToggleSettings),
            Length::Shrink,
            scale,
            palette,
        ),
    ]
    .align_y(Alignment::Center);
    let intro = column![
        page_actions,
        label(
            crate::i18n::tr_args!("settings-studio-title", mode => mode),
            31.0,
            scale,
            with_alpha(palette.surface_text, GLYPH_FADE)
        ),
        label(
            tr("settings-studio-intro"),
            TYPE_SMALL,
            scale,
            with_alpha(palette.surface_text, 0.58),
        )
        .line_height(iced::widget::text::LineHeight::Relative(1.35)),
    ]
    .spacing(7.0 * scale);
    let scroll = scrollable(controls.padding(Padding {
        top: 1.0,
        right: 10.0 * scale,
        bottom: 20.0 * scale,
        left: 0.0,
    }))
    .direction(crate::frontend::ui::thin_vbar())
    .style(crate::frontend::ui::scroll_style(with_alpha(palette.outline, 0.72)))
    .height(Length::Fill);
    let page = column![intro, pinned, scroll]
        .spacing(15.0 * scale)
        .height(Length::Fill)
        .padding([17.0 * scale, 19.0 * scale]);
    let surface = stack![
        container(text("")).width(Length::Fill).height(Length::Fill).style(move |_| {
            crate::frontend::ui::bg_style(Background::Color(with_alpha(
                palette.surface,
                0.985 * reveal,
            )))
        }),
        iced::widget::canvas(Blueprint { palette: *palette, fade: 0.55 * reveal })
            .width(Length::Fill)
            .height(Length::Fill),
        page,
    ]
    .width(Length::Fill)
    .height(Length::Fill);
    let panel = mouse_area(
        container(surface)
            .width(Length::Fixed(panel_width))
            .height(Length::Fixed(panel_height))
            .clip(true)
            .style(move |_| {
                let mut style = crate::frontend::ui::box_style(
                    with_alpha(palette.surface, 0.99 * reveal),
                    with_alpha(palette.outline, 0.72 * reveal),
                );
                style.shadow = iced::Shadow {
                    color: with_alpha(Color::BLACK, 0.4 * reveal),
                    offset: iced::Vector::new(8.0, 14.0),
                    blur_radius: 36.0,
                };
                style
            }),
    )
    .on_press(Message::Noop);
    let (align_x, align_y) = super::super::picker_layout_studio_align(&window, rtl);
    let side = super::super::STUDIO_EDGE_GAP * scale;
    let ends = if window.has_height() { window.edge_gap() } else { 0.0 };
    let on_right = align_x == iced::alignment::Horizontal::Right;
    container(panel)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(align_x)
        .align_y(align_y)
        .padding(Padding {
            top: ends,
            right: if on_right { side } else { 0.0 },
            bottom: ends,
            left: if on_right { 0.0 } else { side },
        })
        .into()
}

fn picker_layout_field<'a>(
    setting: Row,
    index: usize,
    values: &'a HashMap<String, String>,
    has_selected_preset: bool,
    focus: FocusCtx<'_>,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    if matches!(setting.control, Control::AppTheme { .. }) {
        return field(
            setting,
            index,
            values,
            has_selected_preset,
            focus,
            available_width,
            scale,
            palette,
            fade,
            motion,
        );
    }
    if setting.control.is_inline_editor() {
        let (focused, focused_choice) = focus.on(SettingsFocus::Controls, index);
        return compact_inline_field(
            setting,
            index,
            values,
            has_selected_preset,
            focused,
            focused_choice,
            available_width,
            scale,
            palette,
            fade,
            motion,
        );
    }
    if setting.control.is_group_editor() {
        return compact_group_field(
            setting,
            index,
            values,
            has_selected_preset,
            focus,
            available_width,
            scale,
            palette,
            fade,
            motion,
        );
    }
    let (focused, focused_choice) = focus.on(SettingsFocus::Controls, index);
    if setting.control.is_compact_action() {
        return compact_action_field(
            setting,
            index,
            focused,
            available_width,
            scale,
            palette,
            fade,
        );
    }
    let locked = focus.active_input.is_some_and(|key| match &setting.control {
        Control::Number { key: control_key, .. }
        | Control::TextField { key: control_key, .. }
        | Control::Resolution { key: control_key, .. } => control_key == key,
        _ => false,
    });
    let title = if setting.title.is_empty() {
        tr("settings-selector-saved-styles-label").to_string()
    } else {
        setting.title
    };
    let widget = control::widget(
        setting.control,
        values,
        has_selected_preset,
        focused,
        focused_choice,
        available_width,
        scale * 0.94,
        palette,
        fade,
        focus.armed,
        motion,
        Some(focus.reveals()),
    );
    let line = if focused || locked { palette.primary } else { palette.outline };
    mouse_area(
        container(
            column![
                container(text(""))
                    .width(Length::Fill)
                    .height(Length::Fixed(if focused || locked { 2.0 } else { 1.0 }))
                    .style(move |_| crate::frontend::ui::bg_style(Background::Color(with_alpha(
                        line,
                        if focused || locked { 0.9 } else { 0.42 },
                    )))),
                label(title, 13.0, scale, with_alpha(palette.surface_text, fade)),
                label(setting.desc, TYPE_SMALL, scale, with_alpha(palette.surface_text, 0.54))
                    .line_height(iced::widget::text::LineHeight::Relative(1.3)),
                container(widget).padding(Padding { top: 4.0 * scale, ..Padding::ZERO }),
            ]
            .spacing(7.0 * scale),
        )
        .width(Length::Fill)
        .padding([0.0, 3.0 * scale]),
    )
    .on_press(Message::Settings(SettingsMsg::FocusControl(index)))
    .into()
}

/// Custom build (M3): id of the Settings reading pane's scrollable, so a
/// picker command (`skwd-helm ui settings-scroll <px>`) can scroll it.
pub const READING_PANE: &str = "folio.settings.reading";

fn compact_field_copy(
    title: &str,
    desc: &str,
    copy_width: f32,
    scale: f32,
    palette: &Palette,
    fade: f32,
) -> Element<'static, Message> {
    let title = label(title.trim().to_owned(), 12.0, scale, with_alpha(palette.surface_text, fade))
        .width(Length::Fixed(copy_width))
        .wrapping(iced::widget::text::Wrapping::WordOrGlyph);
    if desc.trim().is_empty() {
        title.into()
    } else {
        column![
            title,
            label(
                desc.trim().to_owned(),
                TYPE_SMALL,
                scale,
                with_alpha(palette.surface_text, 0.52 * fade)
            )
            .width(Length::Fixed(copy_width))
            .line_height(iced::widget::text::LineHeight::Relative(1.12))
            .wrapping(iced::widget::text::Wrapping::WordOrGlyph),
        ]
        .spacing(2.0 * scale)
        .into()
    }
}

#[allow(clippy::too_many_arguments)]
fn compact_inline_field<'a>(
    setting: Row,
    index: usize,
    values: &'a HashMap<String, String>,
    has_selected_preset: bool,
    focused: bool,
    focused_choice: Option<usize>,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    let Row { title, desc, control } = setting;
    let editor_width = match &control {
        Control::Number { .. } => (available_width * 0.34).clamp(118.0 * scale, 176.0 * scale),
        Control::TextField { .. } => (available_width * 0.42).clamp(142.0 * scale, 248.0 * scale),
        Control::KeyBinding { .. } => (available_width * 0.5).clamp(176.0 * scale, 300.0 * scale),
        _ => unreachable!("compact inline field requires one scalar editor"),
    };
    let copy_width = (available_width - editor_width - 44.0 * scale).max(72.0 * scale);
    let copy = compact_field_copy(&title, &desc, copy_width, scale, palette, fade);
    let editor = control::widget(
        control,
        values,
        has_selected_preset,
        focused,
        focused_choice,
        editor_width,
        scale * 0.78,
        palette,
        fade,
        None,
        motion,
        None,
    );
    crate::frontend::ui::folio_inline_bar(
        copy,
        focused,
        Message::Settings(SettingsMsg::FocusControl(index)),
        container(editor).width(Length::Fixed(editor_width)).into(),
        scale,
        palette,
        fade,
    )
}

#[allow(clippy::too_many_arguments)]
fn compact_action_field(
    setting: Row,
    index: usize,
    focused: bool,
    available_width: f32,
    scale: f32,
    palette: &Palette,
    fade: f32,
) -> Element<'_, Message> {
    let Row { title, desc, control } = setting;
    let (value, message, enabled, disabled) = match control {
        Control::Toggle { path, value } => (
            value,
            SettingsMsg::Toggle(path, !value),
            tr("settings-control-enabled"),
            tr("settings-control-disabled"),
        ),
        Control::ToggleAction { id, value } => (
            value,
            SettingsMsg::Run(id),
            tr("settings-app-themes-on"),
            tr("settings-app-themes-off"),
        ),
        _ => unreachable!("compact action field requires a toggle"),
    };
    let action_scale = scale * 0.82;
    let action_width = crate::frontend::ui::folio_action_width(enabled, action_scale)
        .max(crate::frontend::ui::folio_action_width(disabled, action_scale));
    let copy_width = (available_width - action_width - 44.0 * scale).max(72.0 * scale);
    let copy = compact_field_copy(&title, &desc, copy_width, scale, palette, fade);
    let action = crate::frontend::ui::folio_action(
        if value { enabled } else { disabled },
        value,
        Some(Message::Settings(message)),
        Length::Fixed(action_width),
        action_scale,
        palette,
    );
    crate::frontend::ui::folio_action_bar(
        copy,
        focused,
        Message::Settings(SettingsMsg::FocusControl(index)),
        action,
        scale,
        palette,
        fade,
    )
}

#[allow(clippy::too_many_arguments)]
fn compact_group_field<'a>(
    setting: Row,
    index: usize,
    values: &'a HashMap<String, String>,
    has_selected_preset: bool,
    focus: FocusCtx<'_>,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    let Row { title, desc, control } = setting;
    let title = if title.is_empty() {
        tr("settings-selector-saved-styles-label").to_string()
    } else {
        title
    };
    let id = control.compact_bar_id().expect("compact group requires a bar identity");
    let (expanded, reveal) = focus.bar(&id);
    let (focused, _) = focus.on(SettingsFocus::Controls, index);
    let is_stack_bar = matches!(&control, Control::StackBar { .. });
    let description_in_tray = !is_stack_bar;
    let summary = match &control {
        Control::MotionWeights { weights } => {
            weights
                .iter()
                .map(|(_, key, _)| values.get(key).map_or("0", String::as_str))
                .collect::<Vec<_>>()
                .join(" / ")
                + " ms"
        }
        Control::StackBar { summary, .. } => summary.clone(),
        _ => unreachable!("compact group requires multiple editors"),
    };
    let content: Element<'a, Message> = if let Control::StackBar { preview, .. } = &control {
        let preview_width = 86.0 * scale;
        let preview_height = 48.0 * scale;
        let identity_width = (available_width * 0.24).clamp(110.0 * scale, 185.0 * scale);
        let wallpaper_width =
            (available_width - preview_width - identity_width - 96.0 * scale).max(100.0 * scale);
        let preview: Element<'a, Message> = preview.as_ref().map_or_else(
            || {
                container(label("◇", 11.0, scale, with_alpha(palette.surface_text, 0.36 * fade)))
                    .width(Length::Fixed(preview_width))
                    .height(Length::Fixed(preview_height))
                    .center(Length::Fill)
                    .style(move |_| {
                        crate::frontend::ui::box_style(
                            with_alpha(palette.surface_container, 0.32 * fade),
                            with_alpha(palette.outline, 0.34 * fade),
                        )
                    })
                    .into()
            },
            |path| {
                container(
                    image(image::Handle::from_path(path))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .content_fit(iced::ContentFit::Cover),
                )
                .width(Length::Fixed(preview_width))
                .height(Length::Fixed(preview_height))
                .clip(true)
                .style(move |_| {
                    crate::frontend::ui::box_style(
                        with_alpha(palette.surface_container, 0.32 * fade),
                        with_alpha(palette.outline, 0.52 * fade),
                    )
                })
                .into()
            },
        );
        let title = crate::frontend::ui::ellipsize_text(
            title.trim(),
            12.5 * legible_type_scale(scale),
            identity_width,
        );
        let desc = crate::frontend::ui::ellipsize_text(
            desc.trim(),
            TYPE_SMALL * legible_type_scale(scale),
            identity_width,
        );
        let summary = crate::frontend::ui::ellipsize_text(
            summary.trim(),
            TYPE_SMALL * legible_type_scale(scale),
            wallpaper_width,
        );
        row![
            preview,
            column![
                label(title, 12.5, scale, with_alpha(palette.surface_text, fade)),
                label(desc, TYPE_SMALL, scale, with_alpha(palette.surface_text, 0.52 * fade)),
            ]
            .width(Length::Fixed(identity_width))
            .spacing(1.0 * scale),
            container(text("")).width(Length::Fill),
            label(summary, TYPE_SMALL, scale, with_alpha(palette.primary, fade))
                .width(Length::Fixed(wallpaper_width))
                .wrapping(iced::widget::text::Wrapping::None)
                .align_x(end()),
        ]
        .spacing(12.0 * scale)
        .align_y(Alignment::Center)
        .into()
    } else {
        row![
            label(title, 12.5, scale, with_alpha(palette.surface_text, fade)),
            container(text("")).width(Length::Fill),
            label(summary, TYPE_SMALL, scale, with_alpha(palette.primary, fade),)
                .width(Length::Fixed((available_width * 0.42).max(90.0 * scale)))
                .wrapping(iced::widget::text::Wrapping::None)
                .align_x(end()),
        ]
        .spacing(10.0 * scale)
        .align_y(Alignment::Center)
        .into()
    };
    let (editor, options_height) = match control {
        Control::StackBar { rows, .. } => {
            let live_height =
                rows.iter().filter(|row| matches!(row.control, Control::Display(_))).count() as f32
                    * 48.0;
            let background_height: f32 = rows
                .iter()
                .filter_map(|row| {
                    let Control::Chips { path, current, background: Some(background), .. } =
                        &row.control
                    else {
                        return None;
                    };
                    let reveal = focus.bar_reveals.get(&format!("background:{path}")).map_or(
                        if matches!(current.as_str(), "center" | "fit") { 1.0 } else { 0.0 },
                        |value| value.x,
                    );
                    let picker =
                        focus.bar_reveals.get(&background.picker_id()).map_or(0.0, |value| value.x);
                    Some((52.0 + 192.0 * picker) * crate::frontend::animation::smoothstep(reveal))
                })
                .sum();
            (
                detail_rows(
                    rows,
                    index,
                    values,
                    has_selected_preset,
                    available_width - 18.0 * scale,
                    scale,
                    palette,
                    fade,
                    focus,
                    motion,
                ),
                176.0 + background_height + live_height,
            )
        }
        control => (
            control::widget(
                control,
                values,
                has_selected_preset,
                focused,
                None,
                available_width - 42.0 * scale,
                scale,
                palette,
                fade,
                focus.armed,
                motion,
                Some(focus.reveals()),
            ),
            110.0,
        ),
    };
    let options: Element<'a, Message> = if desc.trim().is_empty() || !description_in_tray {
        editor
    } else {
        column![
            label(desc, TYPE_SMALL, scale, with_alpha(palette.surface_text, 0.58 * fade),),
            editor,
        ]
        .spacing(6.0 * scale)
        .into()
    };
    crate::frontend::ui::folio_stack_bar(
        content,
        expanded,
        expanded,
        reveal,
        Message::Settings(SettingsMsg::ToggleBar(id, index)),
        None,
        options,
        if is_stack_bar { 68.0 } else { 42.0 },
        options_height,
        scale,
        palette,
    )
}

pub(super) fn field<'a>(
    setting: Row,
    index: usize,
    values: &'a HashMap<String, String>,
    has_selected_preset: bool,
    focus: FocusCtx<'_>,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    motion: MotionProfile,
) -> Element<'a, Message> {
    if let Control::AppTheme { app, index: app_index } = &setting.control {
        return super::app_theme::view(
            app.clone(),
            super::app_theme::Position { row: index, app: *app_index },
            focus.expanded_details.contains(&format!("app-theme:{}", app.id)),
            focus.on(SettingsFocus::Controls, index).0,
            scale,
            palette,
        );
    }
    if setting.control.is_inline_editor() {
        let (focused, focused_choice) = focus.on(SettingsFocus::Controls, index);
        return compact_inline_field(
            setting,
            index,
            values,
            has_selected_preset,
            focused,
            focused_choice,
            available_width,
            scale,
            palette,
            fade,
            motion,
        );
    }
    if setting.control.is_group_editor() {
        return compact_group_field(
            setting,
            index,
            values,
            has_selected_preset,
            focus,
            available_width,
            scale,
            palette,
            fade,
            motion,
        );
    }
    let (focused, focused_choice) = focus.on(SettingsFocus::Controls, index);
    if setting.control.is_compact_action() {
        return compact_action_field(
            setting,
            index,
            focused,
            available_width,
            scale,
            palette,
            fade,
        );
    }
    let Row { title, desc, control } = setting;
    if matches!(control, Control::Segment) {
        return column![
            text(title)
                .font(crate::frontend::ui::UI_FONT)
                .size(22.0 * legible_type_scale(scale))
                .color(with_alpha(palette.surface_text, fade)),
            label(desc, TYPE_SMALL, scale, with_alpha(palette.surface_text, 0.56 * fade))
                .line_height(iced::widget::text::LineHeight::Relative(1.38)),
            folio_horizontal_rule(with_alpha(palette.outline, FOLIO_RULE_ALPHA * fade)),
        ]
        .spacing(9.0 * scale)
        .padding(Padding { top: 10.0 * scale, ..Padding::ZERO })
        .into();
    }
    if let Control::Details { id, summary, rows } = control {
        let expanded = focus.expanded_details.contains(&id);
        let body = detail_rows(
            rows,
            index,
            values,
            has_selected_preset,
            available_width,
            scale,
            palette,
            fade,
            focus,
            motion,
        );
        return crate::frontend::ui::folio_details(
            title,
            summary,
            desc,
            expanded,
            focused,
            Message::Settings(SettingsMsg::ToggleDetails(id, index)),
            body,
            scale,
            palette,
        );
    }
    let locked = focus.active_input.is_some_and(|key| match &control {
        Control::Number { key: control_key, .. }
        | Control::TextField { key: control_key, .. }
        | Control::Resolution { key: control_key, .. } => control_key == key,
        Control::MotionWeights { weights } => {
            weights.iter().any(|(_, control_key, _)| control_key == key)
        }
        _ => false,
    }) || focused && focused_choice.is_some();
    let title = if title.is_empty() {
        tr("settings-selector-saved-styles-label").to_string()
    } else {
        title
    };
    let widget = control::widget(
        control,
        values,
        has_selected_preset,
        focused,
        focused_choice,
        available_width,
        scale,
        palette,
        fade,
        focus.armed,
        motion,
        Some(focus.reveals()),
    );
    let body = column![
        container(text(""))
            .width(Length::Fill)
            .height(Length::Fixed(if focused || locked { 2.0 } else { 1.0 }))
            .style(move |_| crate::frontend::ui::bg_style(Background::Color(
                if focused || locked {
                    with_alpha(palette.primary, fade)
                } else {
                    with_alpha(palette.outline, 0.48 * fade)
                }
            ))),
        label(title, 14.0, scale, with_alpha(palette.surface_text, fade)),
        label(desc, TYPE_SMALL, scale, with_alpha(palette.surface_text, 0.56 * fade))
            .line_height(iced::widget::text::LineHeight::Relative(1.38)),
        container(widget).width(Length::Fill).padding(Padding {
            top: 7.0 * scale,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }),
    ]
    .spacing(8.0 * scale);
    mouse_area(container(body).width(Length::Fill).padding(Padding {
        top: 0.0,
        right: 6.0 * scale,
        bottom: 4.0 * scale,
        left: 6.0 * scale,
    }))
    .on_press(Message::Settings(SettingsMsg::FocusControl(index)))
    .into()
}

#[allow(clippy::too_many_arguments)]
#[expect(clippy::too_many_lines, reason = "composes nested settings rows and app theme controls")]
fn detail_rows<'a>(
    rows: Vec<Row>,
    parent_index: usize,
    values: &'a HashMap<String, String>,
    has_selected_preset: bool,
    available_width: f32,
    scale: f32,
    palette: &'a Palette,
    fade: f32,
    focus: FocusCtx<'_>,
    motion: MotionProfile,
) -> Element<'a, Message> {
    let mut content = column![].spacing(15.0 * scale);
    for row in rows {
        if matches!(row.control, Control::AppTheme { .. }) {
            content = content.push(field(
                row,
                parent_index,
                values,
                has_selected_preset,
                focus,
                available_width,
                scale,
                palette,
                fade,
                motion,
            ));
            continue;
        }
        if let Control::Display(control) = row.control {
            content = content.push(control.view(scale * 0.94, palette));
            continue;
        }
        if let Control::Details { id, summary, rows } = row.control {
            let expanded = focus.expanded_details.contains(&id);
            let body = detail_rows(
                rows,
                parent_index,
                values,
                has_selected_preset,
                (available_width - 24.0 * scale).max(160.0 * scale),
                scale * 0.94,
                palette,
                fade,
                focus,
                motion,
            );
            content = content.push(crate::frontend::ui::folio_details(
                row.title,
                summary,
                row.desc,
                expanded,
                false,
                Message::Settings(SettingsMsg::ToggleDetails(id, parent_index)),
                body,
                scale * 0.94,
                palette,
            ));
            continue;
        }
        if row.control.is_inline_editor() {
            content = content.push(compact_inline_field(
                row,
                parent_index,
                values,
                has_selected_preset,
                false,
                None,
                (available_width - 24.0 * scale).max(160.0 * scale),
                scale * 0.94,
                palette,
                fade,
                motion,
            ));
            continue;
        }
        if row.control.is_group_editor() {
            let nested_focus =
                FocusCtx { focused_control: usize::MAX, focused_choice: None, ..focus };
            content = content.push(compact_group_field(
                row,
                parent_index,
                values,
                has_selected_preset,
                nested_focus,
                (available_width - 24.0 * scale).max(160.0 * scale),
                scale * 0.94,
                palette,
                fade,
                motion,
            ));
            continue;
        }
        if row.control.is_compact_action() {
            content = content.push(compact_action_field(
                row,
                parent_index,
                false,
                (available_width - 24.0 * scale).max(160.0 * scale),
                scale * 0.94,
                palette,
                fade,
            ));
            continue;
        }
        let Row { title, desc, control } = row;
        let widget = control::widget(
            control,
            values,
            has_selected_preset,
            false,
            None,
            (available_width - 24.0 * scale).max(160.0 * scale),
            scale * 0.94,
            palette,
            fade,
            focus.armed,
            motion,
            Some(focus.reveals()),
        );
        content = content.push(
            column![
                label(title, 12.0, scale, with_alpha(palette.surface_text, fade)),
                label(desc, TYPE_SMALL, scale, with_alpha(palette.surface_text, 0.5 * fade))
                    .line_height(iced::widget::text::LineHeight::Relative(1.32)),
                container(widget)
                    .width(Length::Fill)
                    .padding(Padding { top: 3.0 * scale, ..Padding::ZERO }),
            ]
            .spacing(6.0 * scale),
        );
    }
    content.into()
}

fn tracker_position(active: usize, count: usize) -> f32 {
    if count <= 1 { 0.0 } else { active.min(count - 1) as f32 / (count - 1) as f32 }
}

struct IndexRail {
    active: usize,
    count: usize,
    notches: bool,
    palette: Palette,
    fade: f32,
}

impl canvas::Program<Message> for IndexRail {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let x = bounds.width * 0.5;
        let top = 8.0;
        let bottom = (bounds.height - 8.0).max(top);
        frame.stroke(
            &Path::line(Point::new(x, top), Point::new(x, bottom)),
            Stroke::default()
                .with_color(with_alpha(self.palette.outline, 0.48 * self.fade))
                .with_width(1.0),
        );
        if self.notches {
            for index in 0..self.count {
                let y = top + (bottom - top) * tracker_position(index, self.count);
                frame.stroke(
                    &Path::line(Point::new(x - 2.5, y), Point::new(x + 2.5, y)),
                    Stroke::default()
                        .with_color(with_alpha(self.palette.outline, 0.58 * self.fade))
                        .with_width(1.0),
                );
            }
        }
        let y = top + (bottom - top) * tracker_position(self.active, self.count);
        let diamond = crate::frontend::ui::diamond(Point::new(x, y), 5.0);
        frame.fill(&diamond, with_alpha(self.palette.background, self.fade));
        frame.stroke(
            &diamond,
            Stroke::default()
                .with_color(with_alpha(self.palette.primary, self.fade))
                .with_width(1.5),
        );
        vec![frame.into_geometry()]
    }
}

struct Blueprint {
    palette: Palette,
    fade: f32,
}

impl canvas::Program<Message> for Blueprint {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let line = with_alpha(self.palette.outline, 0.075 * self.fade);
        let accent = with_alpha(self.palette.primary, 0.09 * self.fade);
        let centre =
            Point::new(mirror_x(bounds.width * 0.72, 0.0, bounds.width), bounds.height * 0.56);
        for radius in [90.0, 176.0, 264.0] {
            frame.stroke(
                &Path::circle(centre, radius),
                Stroke::default().with_color(line).with_width(1.0),
            );
        }
        frame.stroke(
            &Path::line(Point::new(centre.x, 0.0), Point::new(centre.x, bounds.height)),
            Stroke::default().with_color(line).with_width(1.0),
        );
        frame.stroke(
            &Path::line(Point::new(0.0, centre.y), Point::new(bounds.width, centre.y)),
            Stroke::default().with_color(line).with_width(1.0),
        );
        let marker = crate::frontend::ui::diamond(centre, 7.0);
        frame.stroke(&marker, Stroke::default().with_color(accent).with_width(1.3));
        vec![frame.into_geometry()]
    }
}
