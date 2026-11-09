use crate::contracts::settings::{SettingsSource, keys, transitions};
use crate::i18n::{tr, tr_args};

use super::super::tables::{SHADER_FAMILIES, SHADER_FAMILY_KEY, shader_family, shaders_in_family};
use super::{Builder, Control};

use super::transition_library::{self as library, transition_combo_name, transition_list};

/// Translated label of an upstream family; the custom build's families
/// (playlist, combo, light, shapes) use their English catalogue label.
fn family_label(family: &str) -> String {
    let key = match family {
        "fade" => "settings-transitions-family-fade",
        "wipe" => "settings-transitions-family-wipe",
        "warp" => "settings-transitions-family-warp",
        "break" => "settings-transitions-family-break",
        "sand" => "settings-transitions-family-sand",
        "random" => "settings-transitions-family-random",
        _ => return catalogue_family(family).map_or("", |(_, label, _)| label).to_string(),
    };
    tr(key).to_string()
}

fn family_desc(family: &str) -> String {
    let key = match family {
        "fade" => "settings-transitions-family-fade-desc",
        "wipe" => "settings-transitions-family-wipe-desc",
        "warp" => "settings-transitions-family-warp-desc",
        "break" => "settings-transitions-family-break-desc",
        "sand" => "settings-transitions-family-sand-desc",
        "random" => "settings-transitions-family-random-desc",
        _ => return catalogue_family(family).map_or("", |(_, _, desc)| desc).to_string(),
    };
    tr(key).to_string()
}

fn catalogue_family(family: &str) -> Option<(&'static str, &'static str, &'static str)> {
    SHADER_FAMILIES.iter().copied().find(|(key, _, _)| *key == family)
}

pub(super) fn pretty(shader: &str) -> String {
    transitions::pretty(shader)
}

fn shader_scope_path(shader: &str) -> String {
    format!("{}.{shader}", keys::transition::SHADER_SCOPES)
}

fn shader_scope(cfg: &dyn SettingsSource, shader: &str) -> String {
    let scope = cfg.text(&shader_scope_path(shader));
    if matches!(scope.as_str(), "all" | "primary") {
        return scope;
    }
    if shader.starts_with("sand-") && cfg.text(keys::transition::SAND_SCOPE) == "primary" {
        return String::from("primary");
    }
    String::from("all")
}

pub(super) fn tab_transitions(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    let enabled = cfg.flag_default_true(keys::transition::ENABLED);
    let current = cfg.text(keys::transition::SHADER);
    let current = if current.is_empty() { String::from("random") } else { current };
    let family = shader_family(&current);

    builder.card(
        tr("settings-transitions-transitions-card"),
        tr("settings-transitions-transitions-card-desc"),
    );
    builder.toggle(
        tr("settings-transitions-enable-label"),
        tr("settings-transitions-enable-desc"),
        keys::transition::ENABLED,
    );
    if !enabled {
        return;
    }
    builder.num(
        tr("settings-transitions-duration-label"),
        tr("settings-transitions-duration-desc"),
        keys::transition::DURATION_MS,
        "ms",
    );

    builder.num(
        tr("settings-transitions-rate-label"),
        tr("settings-transitions-rate-desc"),
        keys::transition::FPS,
        "fps",
    );

    let family_opts: Vec<(String, String)> =
        SHADER_FAMILIES.iter().map(|(key, _, _)| ((*key).to_string(), family_label(key))).collect();
    builder.row(
        tr("settings-transitions-family-label"),
        &family_desc(family),
        Control::Chips {
            path: String::from(SHADER_FAMILY_KEY),
            options: family_opts,
            current: family.to_string(),
            disabled: Vec::new(),
            background: None,
        },
    );
    if family == "playlist" {
        let count = transition_list(&cfg.text(keys::transition::PLAYLIST)).len();
        builder.info(
            "Playing in turns",
            &format!(
                "{count} transition(s) in the playlist. Change the list and the order under \"{}\" below.",
                library::PLAYLIST_CARD
            ),
        );
    } else if family == "combo" && current == "combo" {
        builder.info(
            "Playing a combination",
            &format!(
                "{}. Change it under \"{}\" below.",
                transitions::label_of(&transition_combo_name(cfg)),
                library::COMBO_CARD
            ),
        );
    }
    if !matches!(family, "random" | "playlist") {
        let options: Vec<(String, String)> = shaders_in_family(family)
            .into_iter()
            .map(|name| (name.to_string(), pretty(name)))
            .collect();
        let count = options.len();
        builder.row(
            tr("settings-transitions-shader-label"),
            &tr_args!("settings-transitions-shader-count-desc", count => count),
            Control::Dropdown {
                palettes: Vec::new(),
                path: keys::transition::SHADER.to_string(),
                options,
                current,
            },
        );
    }

    tab_sketch(builder);

    builder.card(
        tr("settings-transitions-preview-card"),
        tr("settings-transitions-preview-card-desc"),
    );
    builder.toggle(
        tr("settings-transitions-show-preview-label"),
        tr("settings-transitions-show-preview-desc"),
        keys::transition::PREVIEW,
    );
    if cfg.flag_default_true(keys::transition::PREVIEW) {
        builder.num(
            tr("settings-transitions-preview-fps-label"),
            tr("settings-transitions-preview-fps-desc"),
            keys::transition::PREVIEW_FPS,
            "fps",
        );
        builder.row("", "", Control::Preview);
    }
    library::tab_library(builder);
    library::tab_playlist(builder);
    library::tab_combo(builder);

    builder.card(tr("settings-transitions-sand-card"), tr("settings-transitions-sand-card-desc"));
    builder.text_field(
        tr("settings-transitions-primary-label"),
        tr("settings-transitions-primary-desc"),
        keys::transition::SAND_PRIMARY,
        "DP-1",
    );
    for shader in shaders_in_family(family) {
        let path = shader_scope_path(shader);
        builder.dropdown_cur(
            &pretty(shader),
            tr("settings-transitions-scope-desc"),
            &path,
            &[
                ("all", tr("settings-transitions-scope-all")),
                ("primary", tr("settings-transitions-scope-primary")),
            ],
            shader_scope(cfg, shader),
        );
    }
    if family != "sand" {
        return;
    }
    let quality = cfg.text(keys::transition::SAND_QUALITY);
    let quality = if quality.is_empty() { String::from("auto") } else { quality };
    builder.dropdown_cur(
        tr("settings-transitions-quality-label"),
        tr("settings-transitions-quality-desc"),
        keys::transition::SAND_QUALITY,
        &[
            ("auto", tr("settings-transitions-quality-auto")),
            ("full", tr("settings-transitions-quality-full")),
            ("low", tr("settings-transitions-quality-low")),
        ],
        quality,
    );
    builder.toggle(
        tr("settings-transitions-sharp-label"),
        tr("settings-transitions-sharp-desc"),
        keys::transition::SAND_SHARP,
    );
}

const SKETCH_ENABLED: &str = keys::transition::SKETCH_ENABLED;
const SKETCH_STYLE: &str = keys::transition::SKETCH_STYLE;
const SKETCH_DURATION: &str = keys::transition::SKETCH_DURATION_MS;
const SKETCH_VIDEOS: &str = keys::transition::SKETCH_VIDEOS;
const SKETCH_SCENES: &str = keys::transition::SKETCH_SCENES;
const ORIGIN: &str = keys::transition::ORIGIN;

const SKETCH_STYLES: [(&str, &str); 7] = [
    ("pencil", "Pencil on paper"),
    ("digital", "Digital screen"),
    ("chalk", "Chalkboard"),
    ("blueprint", "Blueprint"),
    ("ink-brush", "Ink brush"),
    ("charcoal", "Charcoal"),
    ("notebook", "Notebook pen"),
];

const ORIGINS: [(&str, &str); 4] = [
    ("default", "Built-in"),
    ("centre", "Centre"),
    ("random", "Random"),
    ("dense", "Busiest area"),
];

fn sketch_style_desc(style: &str) -> &'static str {
    match style {
        "digital" => "Glowing lines on a dark screen, tinted with the wallpaper's colours.",
        "chalk" => "Grainy white chalk on a green board.",
        "blueprint" => "Crisp white lines over a blue engineering grid.",
        "ink-brush" => "Bold black brush strokes with a soft ink wash on rice paper.",
        "charcoal" => "Thick smudgy strokes and soft shadows on toothy paper.",
        "notebook" => "Blue ballpoint lines and cross-hatching on lined school paper.",
        _ => "Graphite outlines, details and light hatching on cream paper.",
    }
}

fn origin_desc(origin: &str) -> &'static str {
    match origin {
        "centre" => {
            "Transitions that grow from a point (iris, ink, ripples, sand swirls) start in the middle of the screen."
        }
        "random" => "Every transition starts from a different random spot.",
        "dense" => {
            "Transitions start where the new wallpaper has the most detail, which is also where the sketch starts drawing."
        }
        _ => "Each transition uses its own starting spot, like the official app.",
    }
}

fn chips(path: &str, options: &[(&str, &str)], current: String) -> Control {
    Control::Chips {
        path: String::from(path),
        options: options
            .iter()
            .map(|(key, label)| ((*key).to_string(), (*label).to_string()))
            .collect(),
        current,
        disabled: Vec::new(),
        background: None,
    }
}

fn known_or(value: String, options: &[(&str, &str)], fallback: &str) -> String {
    if options.iter().any(|(key, _)| *key == value) { value } else { String::from(fallback) }
}

fn tab_sketch(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    builder.card(
        "Sketch before transition",
        "Draw the new wallpaper as a sketch first, then run the transition chosen above from that sketch.",
    );
    let origin = known_or(cfg.text(ORIGIN), &ORIGINS, "default");
    builder.row("Start point", origin_desc(&origin), chips(ORIGIN, &ORIGINS, origin));
    builder.toggle(
        "Draw a sketch first",
        "The outlines, details and shading are drawn stroke by stroke, then your chosen transition (sand, fade, ink, ...) reveals the real wallpaper.",
        SKETCH_ENABLED,
    );
    if !cfg.flag(SKETCH_ENABLED) {
        return;
    }
    let style = known_or(cfg.text(SKETCH_STYLE), &SKETCH_STYLES, "pencil");
    builder.row(
        "Drawing style",
        sketch_style_desc(&style),
        chips(SKETCH_STYLE, &SKETCH_STYLES, style),
    );
    builder.num(
        "Sketch duration",
        "How long the drawing takes, 1000 to 20000 ms (default 6000). Sketch and transition together are capped at 10000 ms.",
        SKETCH_DURATION,
        "ms",
    );
    builder.toggle_default_true(
        "Also sketch videos",
        "Off: video wallpapers use the plain transition. On: the video's first frame is drawn while it waits, then it starts playing.",
        SKETCH_VIDEOS,
    );
    builder.toggle_default_true(
        "Also sketch Wallpaper Engine scenes",
        "Off: scenes use the plain transition.",
        SKETCH_SCENES,
    );
}

mod tests;
