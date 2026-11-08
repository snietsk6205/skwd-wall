use std::time::{Duration, Instant};

use crate::contracts::rendering::{RenderFrame, RenderStats};
use crate::infrastructure::observability::perf::CpuMeter;
use crate::infrastructure::runtime::Reply;

/// Frames the performance overlay looks back on.
const OVERLAY_FRAMES: usize = 240;
/// Frame gaps longer than this are pauses between animations, not frames.
const OVERLAY_PAUSE_MS: f32 = 100.0;

/// Performance overlay and benchmark state.
#[derive(Default)]
pub(crate) struct PerfState {
    pub(crate) cpu: CpuMeter,
    pub(crate) bench: Option<BenchRun>,
    pub(crate) last_report: Option<String>,
    pub(crate) last_report_path: Option<String>,
    /// Render-side frame log, written by the scene renderer.
    pub(crate) render: RenderStats,
    /// `skwd-helm ui bench` connections waiting for the report.
    pub(crate) bench_waiters: Vec<Reply>,
}

impl PerfState {
    /// Called at the end of every frame tick.
    pub(crate) fn after_tick(&mut self, now: Instant, animating: bool) {
        if let Some(bench) = self.bench.as_mut() {
            bench.after_tick(now, animating);
        }
        self.cpu.sample(now, Duration::from_millis(900));
    }

    /// Frames per second and the 50th/95th percentile frame gap of recent
    /// animation, measured where the frames are drawn.
    pub(crate) fn frame_stats(&self) -> (f32, f32, f32) {
        overlay_frame_stats(&self.render.recent(OVERLAY_FRAMES))
    }
}

pub(crate) fn overlay_frame_stats(frames: &[RenderFrame]) -> (f32, f32, f32) {
    let mut gaps: Vec<f32> = frames
        .iter()
        .filter_map(|frame| frame.gap)
        .map(|gap| gap.as_secs_f32() * 1000.0)
        .filter(|gap| *gap < OVERLAY_PAUSE_MS)
        .collect();
    if gaps.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    gaps.sort_by(f32::total_cmp);
    let pick = |fraction: f32| gaps[((gaps.len() - 1) as f32 * fraction).round() as usize];
    let p50 = pick(0.5);
    let fps = if p50 > 0.0 { 1000.0 / p50 } else { 0.0 };
    (fps, p50, pick(0.95))
}

/// One `skwd-helm ui bench` run.
pub(crate) struct BenchRun {
    pub(crate) step: usize,
    pub(crate) repeat: usize,
    pub(crate) started: Instant,
    pub(crate) snapshot: Option<serde_json::Value>,
    pub(crate) mode_before: String,
    pub(crate) phase: Option<BenchPhase>,
    pub(crate) lines: Vec<String>,
    pub(crate) results: Vec<serde_json::Value>,
    /// The user's FPS cap while a phase runs without it.
    pub(crate) fps_before: Option<SavedSetting>,
    /// Frame gaps of the navigation phases, to estimate the screen's refresh interval.
    pub(crate) navigate_gaps_ms: Vec<f32>,
    /// Frames per second while moving: with the user's FPS limit, and without it.
    pub(crate) navigate_fps: (f32, f32),
    /// Plain-language lines for the end of the report.
    pub(crate) summary: Vec<String>,
}

/// A setting the benchmark changed for a while, to put back afterwards.
pub(crate) enum SavedSetting {
    Set(serde_json::Value),
    Unset,
}

/// Something the benchmark did (a key press, a click, a layout switch).
#[derive(Clone, Copy, Debug)]
pub(crate) struct BenchEvent {
    pub(crate) at: Instant,
    pub(crate) label: &'static str,
    /// UI-thread CPU time when it happened.
    pub(crate) cpu: Duration,
}

pub(crate) struct BenchPhase {
    pub(crate) name: &'static str,
    pub(crate) started: Instant,
    pub(crate) cpu_at_start: Duration,
    pub(crate) views_at_start: u64,
    pub(crate) events: Vec<BenchEvent>,
    /// Times the picker had nothing to animate. A frame gap across one of
    /// them is a pause between animations, not a slow frame.
    pub(crate) pauses: Vec<(Instant, Instant)>,
    pub(crate) idle_since: Option<Instant>,
    /// UI-thread CPU time at each frame tick, to tell working from waiting.
    pub(crate) ticks: Vec<(Instant, Duration)>,
}

impl BenchPhase {
    pub(crate) fn new(name: &'static str, cpu_at_start: Duration, views_at_start: u64) -> Self {
        Self {
            name,
            started: Instant::now(),
            cpu_at_start,
            views_at_start,
            events: Vec::new(),
            pauses: Vec::new(),
            idle_since: None,
            ticks: Vec::new(),
        }
    }

    pub(crate) fn after_tick(&mut self, now: Instant, animating: bool, thread_cpu: Duration) {
        self.ticks.push((now, thread_cpu));
        match (animating, self.idle_since) {
            (false, None) => self.idle_since = Some(now),
            (true, Some(since)) => {
                self.pauses.push((since, now));
                self.idle_since = None;
            }
            _ => {}
        }
    }

    /// Every pause, including one still running at `end`.
    pub(crate) fn pauses_until(&self, end: Instant) -> Vec<(Instant, Instant)> {
        let mut pauses = self.pauses.clone();
        if let Some(since) = self.idle_since {
            pauses.push((since, end));
        }
        pauses
    }
}

impl BenchRun {
    pub(crate) fn after_tick(&mut self, now: Instant, animating: bool) {
        if let Some(phase) = self.phase.as_mut() {
            phase.after_tick(
                now,
                animating,
                crate::infrastructure::observability::perf::thread_cpu_time(),
            );
        }
    }
}
