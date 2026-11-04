use crate::contracts::settings::{SettingsSource, keys};
use crate::i18n::{tr, tr_args};

use super::super::tables::{SHADER_FAMILIES, SHADER_FAMILY_KEY, shader_family, shaders_in_family};
use super::{Builder, Control};

fn family_label_key(family: &str) -> &'static str {
    match family {
        "fade" => "settings-transitions-family-fade",
        "wipe" => "settings-transitions-family-wipe",
        "warp" => "settings-transitions-family-warp",
        "break" => "settings-transitions-family-break",
        "sand" => "settings-transitions-family-sand",
        _ => "settings-transitions-family-random",
    }
}

fn family_desc_key(family: &str) -> &'static str {
    match family {
        "fade" => "settings-transitions-family-fade-desc",
        "wipe" => "settings-transitions-family-wipe-desc",
        "warp" => "settings-transitions-family-warp-desc",
        "break" => "settings-transitions-family-break-desc",
        "sand" => "settings-transitions-family-sand-desc",
        _ => "settings-transitions-family-random-desc",
    }
}

pub(super) fn pretty(shader: &str) -> String {
    let mut out = String::with_capacity(shader.len());
    for (idx, word) in shader.split('-').enumerate() {
        if idx > 0 {
            out.push(' ');
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    out
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

    let family_opts: Vec<(String, String)> = SHADER_FAMILIES
        .iter()
        .map(|(key, _, _)| ((*key).to_string(), tr(family_label_key(key)).to_string()))
        .collect();
    builder.row(
        tr("settings-transitions-family-label"),
        tr(family_desc_key(family)),
        Control::Chips {
            path: String::from(SHADER_FAMILY_KEY),
            options: family_opts,
            current: family.to_string(),
            disabled: Vec::new(),
            background: None,
        },
    );
    if family != "random" {
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

const SKETCH_ENABLED: &str = "transition.sketchEnabled";
const SKETCH_STYLE: &str = "transition.sketchStyle";
const SKETCH_DURATION: &str = "transition.sketchDurationMs";

const SKETCH_STYLES: [(&str, &str); 7] = [
    ("pencil", "Pencil on paper"),
    ("digital", "Digital screen"),
    ("chalk", "Chalkboard"),
    ("blueprint", "Blueprint"),
    ("ink-brush", "Ink brush"),
    ("charcoal", "Charcoal"),
    ("notebook", "Notebook pen"),
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

fn tab_sketch(builder: &mut Builder<'_>) {
    let cfg = builder.cfg;
    builder.card(
        "Sketch before transition",
        "Draw the new wallpaper as a sketch first, then run the transition chosen above from that sketch.",
    );
    builder.toggle(
        "Draw a sketch first",
        "The outlines, details and shading are drawn stroke by stroke before the transition starts.",
        SKETCH_ENABLED,
    );
    if !cfg.flag(SKETCH_ENABLED) {
        return;
    }
    let style = cfg.text(SKETCH_STYLE);
    let style = if SKETCH_STYLES.iter().any(|(key, _)| *key == style) {
        style
    } else {
        String::from("pencil")
    };
    builder.row(
        "Drawing style",
        sketch_style_desc(&style),
        Control::Chips {
            path: String::from(SKETCH_STYLE),
            options: SKETCH_STYLES
                .iter()
                .map(|(key, label)| ((*key).to_string(), (*label).to_string()))
                .collect(),
            current: style,
            disabled: Vec::new(),
            background: None,
        },
    );
    builder.num(
        "Sketch duration",
        "How long the drawing takes, 1000 to 20000 ms (default 6000). Sketch and transition together are capped at 10000 ms; Sand transitions become an ink-wash reveal.",
        SKETCH_DURATION,
        "ms",
    );
}

mod tests;
