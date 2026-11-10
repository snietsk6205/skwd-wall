//! R1 (custom build): the picture or video behind Settings → Home. A picture
//! (or a video's poster) is decoded and blurred once per change on a worker
//! thread; a video plays through the picker's live-preview helper only while
//! Home is on screen, and stops for Low-power mode, Battery saver, its pause
//! button, and whenever Settings is closed or hidden.

use std::sync::Arc;
use std::time::Instant;

use skwd_config::keys::settings_window as keys;

use crate::app::App;
use crate::contracts::preview::{BufPool, PREVIEW_HEIGHT, PREVIEW_WIDTH};
use crate::domain::library::catalog::WallpaperKind;
use crate::infrastructure::preview::LiveStreamer;

const VIDEO_FPS: u32 = 24;
const VIDEO_EXTENSIONS: [&str; 8] = ["mp4", "mkv", "webm", "mov", "avi", "m4v", "gif", "wmv"];

fn is_video_path(path: &str) -> bool {
    std::path::Path::new(path).extension().and_then(std::ffi::OsStr::to_str).is_some_and(
        |extension| VIDEO_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str()),
    )
}

/// (source, poster picture, is video) Home should show.
pub(crate) fn wanted(app: &App) -> Option<(String, String, bool)> {
    match app.config.str_path(keys::HOME_BACKGROUND).as_str() {
        "none" => None,
        "custom" => {
            let path = app.config.str_path(keys::HOME_BACKGROUND_PATH);
            let path = path.trim();
            if path.is_empty() {
                return None;
            }
            let path = expand_home(path);
            let video = is_video_path(&path);
            let poster =
                if video { thumb_of(app, &path).unwrap_or_default() } else { path.clone() };
            Some((path, poster, video))
        }
        _ => current_wallpaper(app),
    }
}

fn thumb_of(app: &App, path: &str) -> Option<String> {
    app.library_session
        .library
        .catalog()
        .items
        .iter()
        .find(|item| item.path == path)
        .map(|item| item.thumb.clone())
        .filter(|thumb| !thumb.is_empty())
}

fn current_wallpaper(app: &App) -> Option<(String, String, bool)> {
    let output = app.daemon.output_statuses.iter().find(|output| {
        output.connected && !(output.current.is_empty() && output.path.is_empty())
    })?;
    let path = if output.current.is_empty() { output.path.clone() } else { output.current.clone() };
    let item = app.library_session.library.catalog().items.iter().find(|item| item.path == path);
    let video = item.map_or_else(|| is_video_path(&path), |item| item.kind == WallpaperKind::Video);
    if video {
        let poster = item.map(|item| item.thumb.clone()).unwrap_or_default();
        return Some((path, poster, true));
    }
    if std::path::Path::new(&path).is_file() {
        return Some((path.clone(), path, false));
    }
    // A Wallpaper Engine scene or a missing file: its preview picture.
    let preview = app.daemon.output_wallpaper_art.get(&output.name).cloned()?;
    Some((preview.clone(), preview, false))
}

/// Why a video background is held right now, if it is.
pub(crate) fn held_by(app: &App) -> Option<&'static str> {
    if app.config.low_power() {
        Some("Low-power mode")
    } else if app.config.battery_saver_active() {
        Some("Battery saver")
    } else {
        None
    }
}

fn plays(app: &App) -> bool {
    let nav = &app.panels.settings.nav;
    app.panels.settings.open
        && !app.runtime_state.stashed
        && nav.page.is_none()
        && nav.home.video
        && held_by(app).is_none()
        && !app.config.flag_default_config(keys::HOME_PAUSED)
}

/// Brings the background in line with the settings (cheap when nothing
/// changed). Called after every update while Settings is open.
pub(crate) fn sync(app: &mut App) {
    if !app.panels.settings.open {
        let home = &mut app.panels.settings.nav.home;
        if home.streamer.is_some() || home.frame.is_some() {
            home.streamer = None;
            home.frame = None;
        }
        return;
    }
    let signature = signature(app);
    if app.panels.settings.nav.home.signature != signature {
        app.panels.settings.nav.home.signature = signature;
        let wanted = wanted(app);
        adopt(app, wanted.as_ref());
    }
    if plays(app) {
        let video = app.panels.settings.nav.home.source.clone();
        let home = &mut app.panels.settings.nav.home;
        if home.streamer.is_none() {
            home.streamer = LiveStreamer::start(
                app.runtime_state.wake_tx.clone(),
                Instant::now(),
                Arc::new(BufPool::new((PREVIEW_WIDTH * PREVIEW_HEIGHT * 4) as usize, 3)),
                VIDEO_FPS,
            );
        }
        if let Some(streamer) = home.streamer.as_mut()
            && !streamer.set_path(&video)
        {
            home.streamer = None;
        }
    } else if let Some(streamer) = app.panels.settings.nav.home.streamer.as_mut() {
        streamer.pause();
    }
}

/// What `wanted` depends on, cheap to build (no library scan, no disk).
fn signature(app: &App) -> String {
    let output =
        app.daemon.output_statuses.iter().find(|output| {
            output.connected && !(output.current.is_empty() && output.path.is_empty())
        });
    format!(
        "{}|{}|{}|{}|{}",
        app.config.str_path(keys::HOME_BACKGROUND),
        app.config.str_path(keys::HOME_BACKGROUND_PATH),
        output.map_or("", |output| output.current.as_str()),
        output.map_or("", |output| output.path.as_str()),
        app.library_session.library.catalog().items.len(),
    )
}

/// Switches the backdrop to `wanted` (decodes its picture once).
fn adopt(app: &mut App, wanted: Option<&(String, String, bool)>) {
    let source = wanted.map_or(String::new(), |(source, _, _)| source.clone());
    if app.panels.settings.nav.home.source == source {
        return;
    }
    let home = &mut app.panels.settings.nav.home;
    home.source.clone_from(&source);
    home.still = None;
    home.blurred = None;
    home.frame = None;
    home.streamer = None;
    home.video = wanted.is_some_and(|(_, _, video)| *video);
    if let Some((_, poster, _)) = wanted
        && !poster.is_empty()
        && home.decoding != source
    {
        home.decoding.clone_from(&source);
        crate::infrastructure::preview::spawn_home_backdrop(
            source,
            poster.clone(),
            app.runtime_state.wake_tx.clone(),
        );
    }
}

/// Takes the newest video frame, if one arrived (from `Wake::Decoded`).
pub(crate) fn pull_frame(app: &mut App) {
    if !plays(app) {
        return;
    }
    let Some(frame) =
        app.panels.settings.nav.home.streamer.as_ref().and_then(LiveStreamer::latest_frame)
    else {
        return;
    };
    if frame.len() == (PREVIEW_WIDTH * PREVIEW_HEIGHT * 4) as usize {
        app.panels.settings.nav.home.frame =
            Some(iced::advanced::image::Handle::from_rgba(PREVIEW_WIDTH, PREVIEW_HEIGHT, frame));
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn decoded(
    app: &mut App,
    source: &str,
    pixels: Option<(
        crate::infrastructure::preview::BackdropPixels,
        crate::infrastructure::preview::BackdropPixels,
    )>,
) {
    let home = &mut app.panels.settings.nav.home;
    if home.source != source {
        return;
    }
    home.decoding.clear();
    let Some(((sharp, sw, sh), (blurred, bw, bh))) = pixels else {
        return;
    };
    home.still = Some(iced::advanced::image::Handle::from_rgba(sw, sh, sharp));
    home.blurred = Some(iced::advanced::image::Handle::from_rgba(bw, bh, blurred));
}

/// Words for the caption under the Home cards.
pub(crate) fn caption(app: &App) -> String {
    let home = &app.panels.settings.nav.home;
    match app.config.str_path(keys::HOME_BACKGROUND).as_str() {
        "none" => String::from("No background picture (Settings window → Home background)"),
        "custom" if home.source.is_empty() => {
            String::from("Choose a picture or video under Look → Settings window")
        }
        "custom" => format!("Background: {}", file_name(&home.source)),
        _ if home.source.is_empty() => {
            String::from("Background: your current wallpaper (not known yet)")
        }
        _ => format!(
            "Background: your current wallpaper{}",
            if home.video { " (video)" } else { "" }
        ),
    }
}

fn file_name(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or(path)
        .to_string()
}

fn expand_home(path: &str) -> String {
    match path.strip_prefix("~/") {
        Some(rest) => {
            std::env::var("HOME").map_or_else(|_| path.to_string(), |home| format!("{home}/{rest}"))
        }
        None => path.to_string(),
    }
}
