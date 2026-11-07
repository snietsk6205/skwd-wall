//! `skwd-helm ui bench`: a short, scripted run through the picker that
//! measures idle CPU, frame pacing while moving, the cost of a settings click
//! and of a slider drag, and layout switches. The run works on a temporary
//! copy of the config, so nothing the user chose is changed or saved.

use std::fmt::Write as _;
use std::time::{Duration, Instant};

use iced::Task;
use serde_json::{Value, json};

use crate::app::state::{BenchPhase, BenchRun};
use crate::app::{App, Message};
use crate::frontend::settings::SettingsMsg;
use crate::infrastructure::observability::perf;

/// The bench toggles this flag on and off (it only shows or hides a bar button).
const TOGGLE_KEY: &str = "filterBar.show.layout";
/// The bench drags this number back and forth (the Grid selection ring width).
const SLIDER_KEY: &str = "components.wallpaperSelector.gridFocusWidth";
const MODES: &[&str] =
    &["wall", "hex", "coverflow", "showcase", "depth", "collection", "hand", "sandy", "slices"];
const SPANS: &[&str] = &[
    "update.settings",
    "update.frame",
    "update.other",
    "view",
    "view.settings",
    "tick",
    "apply_layout",
    "settings.init_inputs",
    "config.save",
    "config.adopt",
];

#[derive(Clone, Copy)]
enum Step {
    Prepare,
    Begin(&'static str),
    End,
    Command(&'static str),
    Nav(usize),
    Toggle(usize),
    Slider(usize),
    Modes,
    Finish,
}

/// Each step, then how long to wait before the next one (milliseconds).
const SCRIPT: &[(Step, u64)] = &[
    (Step::Prepare, 1500),
    (Step::Begin("idle"), 3000),
    (Step::End, 0),
    (Step::Begin("navigate"), 0),
    (Step::Nav(24), 110),
    (Step::End, 300),
    (Step::Command("tab picker"), 900),
    (Step::Begin("settings-click"), 0),
    (Step::Toggle(12), 150),
    (Step::End, 300),
    (Step::Begin("settings-slider"), 0),
    (Step::Slider(30), 33),
    (Step::End, 300),
    (Step::Command("dismiss"), 700),
    (Step::Begin("layouts"), 0),
    (Step::Modes, 1100),
    (Step::End, 600),
    (Step::Begin("idle-after"), 3000),
    (Step::End, 0),
    (Step::Finish, 0),
];

pub(super) fn start(app: &mut App) -> Task<Message> {
    if app.runtime_state.perf.bench.is_some() {
        log::warn!("bench: already running");
        return Task::none();
    }
    log::info!("bench: starting (about 30 seconds; the picker will move by itself)");
    app.runtime_state.perf.bench = Some(BenchRun {
        step: 0,
        repeat: 0,
        started: Instant::now(),
        snapshot: None,
        mode_before: app.config.selector_mode(),
        phase: None,
        lines: Vec::new(),
        results: Vec::new(),
    });
    app.runtime_state.perf.last_report = None;
    app.runtime_state.perf.last_report_path = None;
    step(app)
}

pub(super) fn step(app: &mut App) -> Task<Message> {
    let Some(bench) = app.runtime_state.perf.bench.as_ref() else { return Task::none() };
    let Some(&(current, wait)) = SCRIPT.get(bench.step) else { return Task::none() };
    let repeat = bench.repeat;
    let (task, done) = run(app, current, repeat);
    let Some(bench) = app.runtime_state.perf.bench.as_mut() else { return task };
    if done {
        bench.step += 1;
        bench.repeat = 0;
    } else {
        bench.repeat += 1;
    }
    if bench.step < SCRIPT.len() {
        schedule(app, wait);
    }
    task
}

fn schedule(app: &App, wait_ms: u64) {
    let wake = app.runtime_state.wake_tx.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(wait_ms));
        let _ =
            wake.unbounded_send(crate::infrastructure::runtime::Wake::Command("bench-step".into()));
    });
}

fn run(app: &mut App, step: Step, repeat: usize) -> (Task<Message>, bool) {
    match step {
        Step::Prepare => {
            let snapshot = app.config.begin_transient();
            if let Some(bench) = app.runtime_state.perf.bench.as_mut() {
                bench.snapshot = Some(snapshot);
            }
            app.close_settings();
            (crate::app::warm::unstash(app), true)
        }
        Step::Begin(name) => {
            perf::reset();
            if let Some(bench) = app.runtime_state.perf.bench.as_mut() {
                bench.phase = Some(BenchPhase {
                    name,
                    started: Instant::now(),
                    cpu_at_start: perf::cpu_time(),
                    views_at_start: crate::app::view::view_count(),
                    last_frame: None,
                    frame_gaps_ms: Vec::new(),
                });
            }
            (Task::none(), true)
        }
        Step::End => {
            end_phase(app);
            (Task::none(), true)
        }
        Step::Command(command) => (super::run_ui_command(app, command), true),
        Step::Nav(count) => {
            let direction = if (repeat / 8).is_multiple_of(2) { "nav right" } else { "nav left" };
            (super::run_ui_command(app, direction), repeat + 1 >= count)
        }
        Step::Toggle(count) => {
            let value = repeat.is_multiple_of(2);
            let message = Message::Settings(SettingsMsg::Toggle(TOGGLE_KEY.into(), !value));
            (super::update(app, message), repeat + 1 >= count)
        }
        Step::Slider(count) => {
            let width = 1 + (repeat % 12).min(12 - repeat % 12);
            let message =
                Message::Settings(SettingsMsg::Input(SLIDER_KEY.into(), width.to_string()));
            (super::update(app, message), repeat + 1 >= count)
        }
        Step::Modes => {
            let mode = MODES.get(repeat).copied().unwrap_or("wall");
            (super::run_ui_command(app, &format!("mode {mode}")), repeat + 1 >= MODES.len())
        }
        Step::Finish => {
            finish(app);
            (Task::none(), true)
        }
    }
}

fn percentile(sorted: &[f32], fraction: f32) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    sorted[((sorted.len() - 1) as f32 * fraction).round() as usize]
}

fn end_phase(app: &mut App) {
    let Some(bench) = app.runtime_state.perf.bench.as_mut() else { return };
    let Some(phase) = bench.phase.take() else { return };
    let wall = phase.started.elapsed();
    let cpu = perf::cpu_percent(phase.cpu_at_start, perf::cpu_time(), wall);
    let views = crate::app::view::view_count().saturating_sub(phase.views_at_start);
    let mut gaps = phase.frame_gaps_ms;
    gaps.sort_by(f32::total_cmp);
    let p50 = percentile(&gaps, 0.5);
    let p95 = percentile(&gaps, 0.95);
    let p99 = percentile(&gaps, 0.99);
    // No measurable frame gaps: nothing animated, or every frame took longer
    // than a quarter of a second (software rendering).
    let smooth = if gaps.is_empty() {
        if views > 2 { 0.0 } else { 100.0 }
    } else {
        gaps.iter().filter(|gap| **gap <= 20.0).count() as f32 / gaps.len() as f32 * 100.0
    };
    let mut spans = serde_json::Map::new();
    let mut span_text = String::new();
    for name in SPANS {
        let stat = perf::stat(name);
        if stat.count == 0 {
            continue;
        }
        spans.insert(
            (*name).into(),
            json!({
                "count": stat.count,
                "mean_ms": round2(stat.mean_ms()),
                "max_ms": round2(stat.max_ms()),
            }),
        );
        let _ = write!(
            span_text,
            "\n      {name:<22} {:>4}x  avg {:>6.2} ms  max {:>6.2} ms",
            stat.count,
            stat.mean_ms(),
            stat.max_ms()
        );
    }
    let line = format!(
        "{:<16} {:>5.1} s  CPU {:>5.1} %  redraws {:>4}  frame gap p50/p95/p99 {:>5.1}/{:>5.1}/{:>5.1} ms  smooth {:>5.1} %{}",
        phase.name,
        wall.as_secs_f32(),
        cpu,
        views,
        p50,
        p95,
        p99,
        smooth,
        span_text
    );
    log::info!("bench: {line}");
    bench.lines.push(line);
    bench.results.push(json!({
        "phase": phase.name,
        "seconds": round2(wall.as_secs_f64()),
        "cpu_percent": round2(cpu),
        "redraws": views,
        "frames": gaps.len(),
        "frame_gap_ms": { "p50": round2(f64::from(p50)), "p95": round2(f64::from(p95)), "p99": round2(f64::from(p99)) },
        "smooth_percent": round2(f64::from(smooth)),
        "spans": Value::Object(spans),
    }));
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn finish(app: &mut App) {
    let Some(mut bench) = app.runtime_state.perf.bench.take() else { return };
    let save = app.config.measure_save();
    if let Some(snapshot) = bench.snapshot.take() {
        app.config.restore_transient(snapshot);
    }
    if app.config.selector_mode() != bench.mode_before {
        log::warn!("bench: mode restore mismatch");
    }
    app.apply_layout();
    app.sync_filter_bar_chrome();
    let graphics = crate::infrastructure::capabilities::graphics_card(
        &crate::rendering::capabilities::WgpuGraphicsProbe,
    );
    let (width, height) = app.scene.viewport;
    let header = format!(
        "skwd-wall custom benchmark\nversion {}  gpu {}  screen {:.0}x{:.0}  ui scale {:.2}  wallpapers {}  low power {}  total {:.1} s\nconfig save (write the whole config.json once): {:.2} ms",
        env!("CARGO_PKG_VERSION"),
        graphics.name,
        width,
        height,
        app.config.ui_scale(),
        app.library_session.filtered.len(),
        if app.config.low_power() { "on" } else { "off" },
        bench.started.elapsed().as_secs_f32(),
        save.as_secs_f64() * 1000.0,
    );
    let mut text = header.clone();
    for line in &bench.lines {
        text.push('\n');
        text.push_str(line);
    }
    text.push_str(
        "\nGoals: idle CPU near 0 % with 0 redraws; smooth >= 95 %; p95 frame gap <= 20 ms; a settings click under 8 ms.",
    );
    let report = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "gpu": graphics.name,
        "screen": [width, height],
        "ui_scale": app.config.ui_scale(),
        "wallpapers": app.library_session.filtered.len(),
        "low_power": app.config.low_power(),
        "config_save_ms": round2(save.as_secs_f64() * 1000.0),
        "phases": bench.results,
    });
    let path = perf::write_report(&app.config.cache_dir(), &text, &report.to_string());
    log::info!("bench: finished\n{text}");
    if let Some(path) = &path {
        log::info!("bench: report saved to {path}");
        app.show_toast(crate::i18n::tr_args!("status-bench-done", path => path.as_str()));
    }
    app.runtime_state.perf.last_report = Some(text);
    app.runtime_state.perf.last_report_path = path;
    app.retick();
}

/// Reply to `skwd-helm ui perf`: live numbers plus the last bench report.
pub(crate) fn perf_json(app: &App) -> String {
    let spans: serde_json::Map<String, Value> = perf::snapshot()
        .into_iter()
        .map(|(name, stat)| {
            (
                name.to_owned(),
                json!({
                    "count": stat.count,
                    "mean_ms": round2(stat.mean_ms()),
                    "max_ms": round2(stat.max_ms()),
                    "last_ms": round2(stat.last_ms()),
                }),
            )
        })
        .collect();
    json!({
        "bench_running": app.runtime_state.perf.bench.is_some(),
        "bench_report": app.runtime_state.perf.last_report,
        "bench_report_path": app.runtime_state.perf.last_report_path,
        "cpu_percent": round2(app.runtime_state.perf.cpu.percent),
        "metrics": app.runtime_state.metrics.report(),
        "views": crate::app::view::view_count(),
        "config_writes": crate::infrastructure::config::writer::writes(),
        "spans": Value::Object(spans),
    })
    .to_string()
}
