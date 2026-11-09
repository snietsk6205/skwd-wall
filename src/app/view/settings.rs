use iced::Element;

use crate::frontend::settings;

#[allow(clippy::wildcard_imports)]
use super::super::*;

pub(super) fn settings_layer(app: &App) -> Element<'_, Message> {
    crate::zone!("settings_layer");
    let _perf = crate::infrastructure::observability::perf::span("view.settings");
    let source = app.settings_source_ctx();
    let focus = settings::FocusCtx {
        keyboard_focus: app.panels.settings.focus,
        focused_control: app.panels.settings.focused_control,
        focused_choice: app.panels.settings.focused_choice,
        expanded_details: &app.panels.settings.expanded_details,
        bar_reveals: &app.panels.settings.bar_reveals,
        active_input: app.panels.settings.input_edit.as_ref().map(|edit| edit.key.as_str()),
        armed: app.panels.settings.armed.as_ref(),
    };
    let chrome = settings::ChromeCtx {
        viewport: app.scene.viewport,
        window: app.sheet_window(),
        rtl: crate::i18n::is_rtl(),
        entrance: app.panels.settings.entrance.x,
        palette: &app.theme.palette,
    };
    if settings::is_picker_layout_section(&app.panels.settings.tab, app.panels.settings.section) {
        return settings::picker_layout_workbench(&source, focus, chrome);
    }
    settings::settings_workbench(settings::WorkbenchInput {
        nav: nav_view(app),
        source,
        focus,
        chrome,
        profiles: settings::ProfilesCtx {
            selected: app.panels.settings.profile,
            filter: &app.panels.settings.profile_filter,
            enter: app.panels.settings.profile_enter.x,
            marker: app.panels.settings.profile_marker.x,
        },
        folders: settings::FoldersCtx {
            drafts: &app.panels.settings.folder_drafts,
            undo: app
                .panels
                .settings
                .folder_undo
                .as_ref()
                .map(|undo| (undo.kind, undo.path.as_str())),
        },
        tab: &app.panels.settings.tab,
        selected_section: app.panels.settings.section,
        keybind_capture: keybind_capture_view(app),
        process_picker: process_picker_view(app),
        weather_review: weather_review_view(app),
    })
}

/// R1: what the redesigned window shows (page, menus, Home, palette, ...).
fn nav_view(app: &App) -> settings::NavView<'_> {
    use crate::app::update::{settings_home, settings_nav};
    use skwd_config::keys::settings_window as keys;
    let nav = &app.panels.settings.nav;
    let still = app.config.low_power();
    let ease = |x: f32| if still { 1.0 } else { x };
    let palette = nav.palette.as_ref().map(|palette| {
        let results = settings_nav::palette_results(app);
        settings::PaletteView {
            query: &palette.query,
            selected: palette.selected.min(results.len().saturating_sub(1)),
            results: results.into_iter().map(|index| &palette.entries[index]).collect(),
            empty_query: palette.query.trim().is_empty(),
        }
    });
    let bubble = nav.bubble.as_ref().map(|bubble| {
        let (title, desc) = transition_caption(app);
        settings::BubbleView {
            row: bubble.row,
            pinned: bubble.pinned,
            title,
            desc,
            frame: app.panels.transition_preview.handle.clone(),
            fade: ease(nav.bubble_anim.x),
        }
    });
    let recent = if nav.page.is_none() { nav.recent.clone() } else { Vec::new() };
    let picture = nav.home.frame.clone().or_else(|| nav.home.still.clone());
    settings::NavView {
        page: nav.page,
        can_go_back: !nav.history.is_empty(),
        menu: nav.menu,
        menu_item: nav.menu_item,
        menu_fade: ease(nav.menu_anim.x),
        menu_unavailable: &nav.menu_unavailable,
        menu_grid: app.config.str_path(keys::MENU_STYLE) == "grid",
        bar_item: nav.bar_item,
        page_fade: ease(nav.page_anim.x),
        simple: settings_nav::simple_view(app),
        hides_advanced: settings_nav::hides_advanced(app),
        edit_cards: nav.edit_cards,
        card_focus: nav.card_focus,
        card_hover: nav.card_hover,
        card_fade: ease(nav.card_anim.x),
        home_cards: settings_nav::home_cards(app),
        recent,
        palette,
        toast: nav.toast.as_ref().map(|toast| (toast.text.clone(), toast.can_undo)),
        toast_fade: ease(nav.toast_anim.x),
        bubble,
        backdrop: settings::Backdrop {
            picture,
            blurred: nav.home.blurred.clone(),
            video: nav.home.video,
            paused: app.config.flag_default_config(keys::HOME_PAUSED),
            held: if nav.home.video { settings_home::held_by(app) } else { None },
            caption: settings_home::caption(app),
        },
        bar_blur: app.config.flag_default_config(keys::BAR_BLUR),
        more: &nav.more,
        playlist: app.config.str_path(skwd_config::keys::transition::PLAYLIST),
        still,
    }
}

/// Custom build (M3): the name and description of what the live preview plays.
fn transition_caption(app: &App) -> (String, String) {
    use crate::contracts::settings::transitions;
    let shader = crate::app::update::preview_shader(app);
    let describe = |name: &str| {
        transitions::spec(name).map_or_else(
            || {
                if name.starts_with(transitions::COMBO_PREFIX) {
                    String::from("A combination of two transitions.")
                } else {
                    String::new()
                }
            },
            |spec| spec.desc.to_string(),
        )
    };
    if app.panels.transition_preview.focus.is_some() {
        return (format!("Previewing: {}", transitions::label_of(&shader)), describe(&shader));
    }
    let chosen = app.config.str_path(skwd_config::keys::transition::SHADER);
    match chosen.as_str() {
        "playlist" => (
            String::from("Your choice: play in turns"),
            format!(
                "The preview shows the first one, {}. Each wallpaper change plays the next.",
                transitions::label_of(&shader)
            ),
        ),
        _ => (format!("Your choice: {}", transitions::label_of(&shader)), describe(&shader)),
    }
}

fn weather_review_view(app: &App) -> Option<settings::WeatherReviewView> {
    let review = &app.panels.settings;
    let category = review.weather_review_category?;
    let approved = crate::app::update::weather_review::approved(app, category);
    let catalog = app.library_session.library.catalog();
    let catalog_by_key = catalog
        .items
        .iter()
        .map(|wallpaper| (wallpaper.key.as_str(), wallpaper))
        .collect::<std::collections::HashMap<_, _>>();
    let items = crate::app::update::weather_review::visible_keys(app)
        .into_iter()
        .map(|key| {
            let wallpaper = catalog_by_key.get(key.as_str()).copied();
            settings::WeatherReviewItem {
                name: wallpaper.map_or_else(|| key.clone(), |wallpaper| wallpaper.name.clone()),
                thumbnail: wallpaper.map_or_else(String::new, |wallpaper| wallpaper.thumb.clone()),
                approved: approved.contains(&key),
                missing: wallpaper.is_none(),
                key,
            }
        })
        .collect();
    Some(settings::WeatherReviewView {
        category: category.to_owned(),
        tab: review.weather_review_tab,
        query: review.weather_review_query.clone(),
        items,
        loading: review.weather_review_loading,
        error: review.weather_review_error.clone(),
        focus: review.weather_review_focus,
        library_limited: review.weather_review_tab == settings::WeatherReviewTab::Library
            && catalog.items.len() > crate::app::update::weather_review::VISIBLE_LIMIT,
    })
}

fn process_picker_view(app: &App) -> Option<settings::ProcessPickerView> {
    app.panels.settings.process_picker_open.then(|| {
        let targets = app
            .config
            .str_path(skwd_config::keys::playback::PROCESSES)
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        let query = app.panels.settings.process_picker_query.to_ascii_lowercase();
        let mut running = app
            .daemon
            .playback
            .available_processes
            .iter()
            .map(|process| process_name(process))
            .filter(|process| !targets.iter().any(|target| target.eq_ignore_ascii_case(process)))
            .filter(|process| query.is_empty() || process.to_ascii_lowercase().contains(&query))
            .collect::<Vec<_>>();
        running.sort_unstable();
        running.dedup();
        settings::ProcessPickerView {
            targets,
            running,
            query: app.panels.settings.process_picker_query.clone(),
            manual: app.panels.settings.process_picker_manual.clone(),
            scroll: app.panels.settings.process_picker_scroll,
        }
    })
}

fn process_name(process: &str) -> String {
    process
        .split_whitespace()
        .next()
        .unwrap_or(process)
        .rsplit('/')
        .next()
        .unwrap_or(process)
        .to_string()
}

fn keybind_capture_view(app: &App) -> Option<settings::KeybindCaptureView> {
    let capture = app.panels.settings.keybind_capture.as_ref()?;
    let conflict = capture
        .triggers
        .iter()
        .find_map(|trigger| {
            app.input
                .bindings
                .trigger_holders(capture.action, trigger)
                .next()
                .map(|other| (other, trigger.label()))
        })
        .and_then(|(other, trigger)| {
            crate::contracts::picker::KEY_BINDINGS
                .into_iter()
                .find(|entry| entry.action == other)
                .map(|entry| {
                    crate::i18n::tr_args!(
                        "settings-keybind-capture-conflict",
                        binding => trigger,
                        action => crate::i18n::tr(entry.title_key)
                    )
                })
        });
    let binding = if capture.triggers.is_empty() {
        crate::i18n::tr("settings-keybind-unbound").to_string()
    } else {
        crate::domain::input::binding_label(&capture.triggers)
    };
    let action = crate::i18n::tr(capture.title_key);
    Some(settings::KeybindCaptureView {
        title: if capture.slot == 0 {
            action.to_string()
        } else {
            crate::i18n::tr_args!("settings-keybind-capture-second-title", action => action)
        },
        binding,
        edited: capture.edited,
        conflict,
    })
}
