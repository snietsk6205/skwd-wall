//! `skwd-helm ui bench`: a short, scripted run through the picker that
//! measures idle redraws, frame pacing while moving, how fast a settings click
//! shows on screen (the new background save and the old way, side by side),
//! slider drags and layout switches. Frame timings come from the renderer's own
//! frame log, so they are the frames really drawn. The run works on a temporary
//! copy of the config, so nothing the user chose is changed or saved.

use std::fmt::Write as _;
use std::time::{Duration, Instant};

use iced::Task;
use serde_json::{Value, json};

use super::report::{
    Latency, SwitchResult, animated_gaps, gap_stats, latency, latency_stats, ms, on_time_share,
    refresh_estimate, round2, switch_json, switch_line, switch_results,
};
use crate::app::state::{BenchEvent, BenchPhase, BenchRun, SavedSetting};
use crate::app::{App, Message};
use crate::frontend::settings::SettingsMsg;
use crate::infrastructure::observability::perf;
use crate::infrastructure::runtime::Reply;

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
    "bench.old_save",
];
/// Frame cap used by the "navigate-uncapped" phase: the screen alone paces it.
const UNCAPPED_FPS: f64 = 1000.0;
/// Frames drawn this soon after a phase starts belong to the bench's own step.
const OWN_STEP: Duration = Duration::from_millis(50);

#[derive(Clone, Copy)]
enum Step {
    Prepare,
    Begin(&'static str),
    End,
    Command(&'static str),
    /// Sets the FPS cap on the temporary config (`None` puts the user's back).
    Fps(Option<f64>),
    Nav(usize),
    /// Settings toggles; `true` also saves the config the old way (on the UI thread).
    Toggle(usize, bool),
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
    (Step::Fps(Some(UNCAPPED_FPS)), 200),
    (Step::Begin("navigate-uncapped"), 0),
    (Step::Nav(24), 110),
    (Step::End, 300),
    (Step::Fps(None), 0),
    (Step::Command("tab picker"), 900),
    (Step::Begin("settings-click"), 0),
    (Step::Toggle(12, false), 150),
    (Step::End, 300),
    (Step::Begin("settings-click-old"), 0),
    (Step::Toggle(12, true), 150),
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

pub(in crate::app::update) fn start(app: &mut App) -> Task<Message> {
    if app.runtime_state.perf.bench.is_some() {
        log::warn!("bench: already running");
        return Task::none();
    }
    log::info!("bench: starting (about 35 seconds; the picker will move by itself)");
    app.runtime_state.perf.bench = Some(BenchRun {
        step: 0,
        repeat: 0,
        started: Instant::now(),
        snapshot: None,
        mode_before: app.config.selector_mode(),
        fps_before: None,
        phase: None,
        lines: Vec::new(),
        results: Vec::new(),
        navigate_gaps_ms: Vec::new(),
        navigate_fps: (0.0, 0.0),
        summary: Vec::new(),
    });
    app.runtime_state.perf.last_report = None;
    app.runtime_state.perf.last_report_path = None;
    step(app)
}

/// `skwd-helm ui bench`: starts a run (unless one is running) and answers
/// `reply` with the report when it ends, so the helper never has to poll.
pub(in crate::app::update) fn start_and_reply(app: &mut App, reply: Reply) -> Task<Message> {
    app.runtime_state.perf.bench_waiters.push(reply);
    if app.runtime_state.perf.bench.is_some() {
        return Task::none();
    }
    start(app)
}

pub(in crate::app::update) fn step(app: &mut App) -> Task<Message> {
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

/// Notes a bench action, so its click-to-frame time can be measured.
fn mark(app: &mut App, label: &'static str) {
    if let Some(phase) =
        app.runtime_state.perf.bench.as_mut().and_then(|bench| bench.phase.as_mut())
    {
        phase.events.push(BenchEvent { at: Instant::now(), label, cpu: perf::thread_cpu_time() });
    }
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
            let phase = BenchPhase::new(name, perf::cpu_time(), crate::app::view::view_count());
            if let Some(bench) = app.runtime_state.perf.bench.as_mut() {
                bench.phase = Some(phase);
            }
            (Task::none(), true)
        }
        Step::End => {
            end_phase(app);
            (Task::none(), true)
        }
        Step::Command(command) => (super::super::run_ui_command(app, command), true),
        Step::Fps(cap) => {
            set_fps_cap(app, cap);
            (Task::none(), true)
        }
        Step::Nav(count) => {
            let direction = if (repeat / 8).is_multiple_of(2) { "nav right" } else { "nav left" };
            mark(app, "key");
            (super::super::run_ui_command(app, direction), repeat + 1 >= count)
        }
        Step::Toggle(count, old_way) => {
            let value = repeat.is_multiple_of(2);
            mark(app, if old_way { "click (old save)" } else { "click" });
            let message = Message::Settings(SettingsMsg::Toggle(TOGGLE_KEY.into(), !value));
            let task = super::super::update(app, message);
            if old_way {
                // Before M1 every settings change wrote config.json here, on the UI
                // thread. Write the same file the same way (to a probe file next to it).
                let saved = app.config.measure_save();
                perf::record("bench.old_save", saved);
            }
            (task, repeat + 1 >= count)
        }
        Step::Slider(count) => {
            let width = 1 + (repeat % 12).min(12 - repeat % 12);
            mark(app, "slider");
            let message =
                Message::Settings(SettingsMsg::Input(SLIDER_KEY.into(), width.to_string()));
            (super::super::update(app, message), repeat + 1 >= count)
        }
        Step::Modes => {
            // Start after the user's layout and end on it, so every step is a real switch.
            let current =
                app.runtime_state.perf.bench.as_ref().and_then(|bench| {
                    crate::contracts::picker::Mode::try_from_key(&bench.mode_before)
                });
            let first = current
                .and_then(|mode| MODES.iter().position(|key| *key == mode.as_key()))
                .map_or(0, |index| index + 1);
            let mode = MODES[(first + repeat) % MODES.len()];
            mark(app, mode);
            (super::super::run_ui_command(app, &format!("mode {mode}")), repeat + 1 >= MODES.len())
        }
        Step::Finish => {
            finish(app);
            (Task::none(), true)
        }
    }
}

fn set_fps_cap(app: &mut App, cap: Option<f64>) {
    let key = skwd_config::keys::general::MAX_FPS;
    let Some(bench) = app.runtime_state.perf.bench.as_mut() else { return };
    match cap {
        Some(cap) => {
            bench.fps_before =
                Some(app.config.value_at(key).map_or(SavedSetting::Unset, SavedSetting::Set));
            app.config.set_key(key, json!(cap));
        }
        None => match bench.fps_before.take() {
            Some(SavedSetting::Set(previous)) => app.config.set_key(key, previous),
            Some(SavedSetting::Unset) => app.config.remove_key(key),
            None => {}
        },
    }
    app.retick();
}

struct PhaseNumbers {
    line: String,
    json: Value,
}

fn end_phase(app: &mut App) {
    let end = Instant::now();
    let render = app.runtime_state.perf.render.clone();
    let Some(bench) = app.runtime_state.perf.bench.as_mut() else { return };
    let Some(phase) = bench.phase.take() else { return };
    let wall = end.saturating_duration_since(phase.started);
    let cpu = perf::cpu_percent(phase.cpu_at_start, perf::cpu_time(), wall);
    // The view right after the bench's own Begin step is not the picker's doing.
    let redraws =
        crate::app::view::view_count().saturating_sub(phase.views_at_start).saturating_sub(1);
    let frames: Vec<_> = render
        .frames_between(phase.started, end)
        .into_iter()
        .filter(|frame| frame.at >= phase.started + OWN_STEP || !phase.events.is_empty())
        .collect();
    let pauses = phase.pauses_until(end);
    let gaps = animated_gaps(&frames, &pauses);
    let stats = gap_stats(&gaps);
    let latencies: Vec<Latency> =
        phase.events.iter().filter_map(|event| latency(event.at, &frames)).collect();
    if phase.name.starts_with("navigate") {
        bench.navigate_gaps_ms.extend(gaps.iter().map(|gap| gap.ms));
        let fps = if stats.p50 > 0.0 { 1000.0 / stats.p50 } else { 0.0 };
        if phase.name == "navigate" {
            bench.navigate_fps.0 = fps;
        } else {
            bench.navigate_fps.1 = fps;
        }
    }
    let numbers = phase_numbers(&phase, wall, cpu, redraws, frames.len(), &stats, &latencies);
    let mut line = numbers.line;
    let mut json = numbers.json;
    let m1 = super::report::tick_gaps_m1(&phase.ticks);
    if !m1.is_empty() {
        let p99 = super::report::percentile(&m1, 0.99);
        let smooth = m1.iter().filter(|gap| **gap <= 20.0).count() as f32 / m1.len() as f32 * 100.0;
        let _ = write!(
            line,
            "\n      the first benchmark's method (gaps between ticks) would say: p99 {p99:.1} ms, smooth {smooth:.1} %"
        );
        json["m1_tick_gaps"] =
            json!({ "p99": round2(f64::from(p99)), "smooth_percent": round2(f64::from(smooth)) });
    }
    if phase.name == "layouts" {
        let switches = switch_results(&phase.events, &frames, &pauses, &phase.ticks, end);
        for result in &switches {
            line.push_str(&switch_line(result));
        }
        json["switches"] = Value::Array(switches.iter().map(switch_json).collect());
        bench.summary.extend(layout_summary(&switches));
    }
    summarize_phase(bench, phase.name, &stats, &latencies);
    log::info!("bench: {line}");
    bench.lines.push(line);
    bench.results.push(json);
}

fn phase_numbers(
    phase: &BenchPhase,
    wall: Duration,
    cpu: f64,
    redraws: u64,
    frames: usize,
    stats: &super::report::GapStats,
    latencies: &[Latency],
) -> PhaseNumbers {
    let seconds = wall.as_secs_f32();
    let mut line = format!(
        "{:<19} {:>5.1} s  CPU {:>5.1} %  redraws {:>4}  frames {:>4}",
        phase.name, seconds, cpu, redraws, frames
    );
    if stats.count > 0 {
        let _ = write!(
            line,
            "  frame gap p50/p95/p99/max {:.1}/{:.1}/{:.1}/{:.1} ms ({:.0} fps)  smooth {:.1} %",
            stats.p50,
            stats.p95,
            stats.p99,
            stats.max,
            if stats.p50 > 0.0 { 1000.0 / stats.p50 } else { 0.0 },
            stats.smooth
        );
    }
    let (latency_p50, latency_p95, latency_max) = latency_stats(latencies);
    if !latencies.is_empty() {
        let _ = write!(
            line,
            "\n      {} -> frame p50/p95/max {latency_p50:.1}/{latency_p95:.1}/{latency_max:.1} ms ({} times)",
            if phase.name == "layouts" {
                "switch"
            } else {
                phase.events.first().map_or("action", |event| event.label)
            },
            latencies.len()
        );
    }
    let mut spans = serde_json::Map::new();
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
            line,
            "\n      {name:<22} {:>4}x  avg {:>6.2} ms  max {:>6.2} ms",
            stat.count,
            stat.mean_ms(),
            stat.max_ms()
        );
    }
    let json = json!({
        "phase": phase.name,
        "seconds": round2(wall.as_secs_f64()),
        "cpu_percent": round2(cpu),
        "redraws": redraws,
        "frames": frames,
        "frame_gap_ms": {
            "count": stats.count,
            "p50": round2(f64::from(stats.p50)),
            "p95": round2(f64::from(stats.p95)),
            "p99": round2(f64::from(stats.p99)),
            "max": round2(f64::from(stats.max)),
        },
        "smooth_percent": round2(f64::from(stats.smooth)),
        "action_to_frame_ms": {
            "count": latencies.len(),
            "p50": round2(f64::from(latency_p50)),
            "p95": round2(f64::from(latency_p95)),
            "max": round2(f64::from(latency_max)),
        },
        "spans": Value::Object(spans),
    });
    PhaseNumbers { line, json }
}

/// Plain-language lines for the end of the report.
fn summarize_phase(
    bench: &mut BenchRun,
    name: &str,
    stats: &super::report::GapStats,
    latencies: &[Latency],
) {
    let (p50, _, max) = latency_stats(latencies);
    match name {
        "navigate" if stats.count > 0 => bench.summary.push(format!(
            "Moving with the arrow keys: {:.0} fps (half the frames {:.1} ms apart or less).",
            1000.0 / stats.p50.max(0.1),
            stats.p50
        )),
        "navigate-uncapped" if stats.count > 0 => bench.summary.push(format!(
            "The same without the FPS cap ({} limit): {:.0} fps.",
            "general.maxFps",
            1000.0 / stats.p50.max(0.1)
        )),
        "settings-click" if !latencies.is_empty() => bench.summary.push(format!(
            "Settings click, new way (saved in the background): shows on screen after {p50:.1} ms (slowest {max:.1} ms)."
        )),
        "settings-click-old" if !latencies.is_empty() => bench.summary.push(format!(
            "Settings click, old way (saved on the UI thread, as before M1): {p50:.1} ms (slowest {max:.1} ms)."
        )),
        _ => {}
    }
}

fn layout_summary(switches: &[SwitchResult]) -> Option<String> {
    let first = switches
        .iter()
        .filter_map(|result| result.first.map(|first| (result.label, first)))
        .max_by(|lhs, rhs| lhs.1.ms.total_cmp(&rhs.1.ms))?;
    let slowest = switches
        .iter()
        .filter_map(|result| result.slowest.map(|gap| (result.label, gap)))
        .max_by(|lhs, rhs| lhs.1.ms.total_cmp(&rhs.1.ms));
    let mut text = format!(
        "Layout switch: the slowest first frame was {:.1} ms ({}; renderer {:.1} ms of it)",
        first.1.ms,
        first.0,
        ms(first.1.frame.work.total())
    );
    if let Some((label, gap)) = slowest {
        let _ = write!(text, "; the slowest frame after a switch {:.1} ms ({label})", gap.ms);
    }
    text.push('.');
    Some(text)
}

fn refresh_summary(bench: &BenchRun, fps_cap: f32) -> Vec<String> {
    let Some(refresh) = refresh_estimate(&bench.navigate_gaps_ms) else { return Vec::new() };
    let hz = 1000.0 / refresh;
    let on_time = on_time_share(&bench.navigate_gaps_ms, refresh);
    let mut lines = vec![format!(
        "Screen: the fastest frames came {refresh:.1} ms apart (about {hz:.0} Hz); {on_time:.0} % of the frames while moving were ready for the next refresh. Each frame (CPU + GPU) must be ready within one refresh, or it waits for the one after (that halves the frame rate)."
    )];
    let capped = bench.navigate_fps.0;
    let uncapped = bench.navigate_fps.1;
    if fps_cap + 1.0 < hz && uncapped > capped * 1.1 {
        lines.push(format!(
            "Advice: the FPS limit ({fps_cap:.0}) is below the screen's {hz:.0} Hz and this computer is fast enough for more: set Settings > Performance > Max FPS to {:.0}.",
            hz.round()
        ));
    } else if on_time < 70.0 {
        lines.push(String::from(
            "Advice: most frames need more than one refresh on this graphics chip, so the FPS limit is not what holds it back. Compare with the other graphics card (skwd-helm ui bench --gpu high) or try low-power mode.",
        ));
    }
    lines
}

fn finish(app: &mut App) {
    let Some(mut bench) = app.runtime_state.perf.bench.take() else { return };
    let save = app.config.measure_save();
    let fps_cap = app.config.max_fps();
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
    let power_preference = std::env::var("WGPU_POWER_PREF").unwrap_or_else(|_| "default".into());
    let present_mode = std::env::var("ICED_PRESENT_MODE").unwrap_or_else(|_| "vsync".into());
    let header = format!(
        "skwd-wall custom benchmark\nversion {}  gpu {}  screen {:.0}x{:.0}  ui scale {:.2}  wallpapers {}  low power {}  fps cap {:.0}  total {:.1} s\nGPU preference {power_preference}  present mode {present_mode}  one config save on this disk: {:.2} ms",
        env!("CARGO_PKG_VERSION"),
        graphics.name,
        width,
        height,
        app.config.ui_scale(),
        app.library_session.filtered.len(),
        if app.config.low_power() { "on" } else { "off" },
        fps_cap,
        bench.started.elapsed().as_secs_f32(),
        save.as_secs_f64() * 1000.0,
    );
    let mut text = header;
    for line in &bench.lines {
        text.push('\n');
        text.push_str(line);
    }
    text.push_str("\n\nIn short:");
    for line in refresh_summary(&bench, fps_cap).iter().chain(bench.summary.iter()) {
        text.push_str("\n- ");
        text.push_str(line);
    }
    text.push_str(
        "\nGoals: idle 0 redraws; smooth >= 95 %; a settings click on screen within 2 refreshes; a layout's first frame under 50 ms.",
    );
    let report = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "gpu": graphics.name,
        "screen": [width, height],
        "ui_scale": app.config.ui_scale(),
        "wallpapers": app.library_session.filtered.len(),
        "low_power": app.config.low_power(),
        "fps_cap": fps_cap,
        "gpu_preference": power_preference,
        "present_mode": present_mode,
        "config_save_ms": round2(save.as_secs_f64() * 1000.0),
        "refresh_ms": refresh_estimate(&bench.navigate_gaps_ms).map(|refresh| round2(f64::from(refresh))),
        "phases": bench.results,
    });
    let path = perf::write_report(&app.config.cache_dir(), &text, &report.to_string());
    log::info!("bench: finished\n{text}");
    if let Some(path) = &path {
        log::info!("bench: report saved to {path}");
        app.show_toast(crate::i18n::tr_args!("status-bench-done", path => path.as_str()));
    }
    let reply = json!({ "report": text, "path": path }).to_string();
    for waiter in app.runtime_state.perf.bench_waiters.drain(..) {
        waiter.send(&reply);
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
    let recent: Vec<Value> = app
        .runtime_state
        .perf
        .render
        .recent(20)
        .iter()
        .map(|frame| {
            json!({
                "gap_ms": frame.gap.map(|gap| round2(gap.as_secs_f64() * 1000.0)),
                "work": super::report::work_json(&frame.work),
            })
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
        "recent_frames": recent,
        "spans": Value::Object(spans),
    })
    .to_string()
}
