//! Numbers for the benchmark report, computed from the render-side frame log
//! (when frames were really drawn and what the renderer did in each) and from
//! the benchmark's own actions. Pure functions, so they can be unit-tested.

use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::app::state::BenchEvent;
use crate::contracts::rendering::{RenderFrame, RenderWork};

/// A frame at most this late still looks smooth (about 50 fps).
pub(super) const SMOOTH_MS: f32 = 20.0;

pub(super) fn ms(duration: Duration) -> f32 {
    duration.as_secs_f32() * 1000.0
}

pub(super) fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

pub(super) fn percentile(sorted: &[f32], fraction: f32) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    sorted[((sorted.len() - 1) as f32 * fraction).round() as usize]
}

/// A gap between two drawn frames while something was animating.
#[derive(Clone, Copy, Debug)]
pub(super) struct Gap {
    pub(super) ms: f32,
    pub(super) before: RenderFrame,
    pub(super) frame: RenderFrame,
}

/// Gaps between consecutive frames. A gap across a pause (nothing was
/// animating, so no frame was due) is left out: it is not a slow frame.
pub(super) fn animated_gaps(frames: &[RenderFrame], pauses: &[(Instant, Instant)]) -> Vec<Gap> {
    frames
        .windows(2)
        .filter_map(|pair| {
            let (before, frame) = (pair[0], pair[1]);
            let paused = pauses.iter().any(|(start, end)| *start < frame.at && *end > before.at);
            (!paused).then(|| Gap { ms: ms(frame.at - before.at), before, frame })
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct GapStats {
    pub(super) count: usize,
    pub(super) p50: f32,
    pub(super) p95: f32,
    pub(super) p99: f32,
    pub(super) max: f32,
    /// Share of gaps no longer than [`SMOOTH_MS`].
    pub(super) smooth: f32,
}

pub(super) fn gap_stats(gaps: &[Gap]) -> GapStats {
    let mut sorted: Vec<f32> = gaps.iter().map(|gap| gap.ms).collect();
    if sorted.is_empty() {
        return GapStats::default();
    }
    sorted.sort_by(f32::total_cmp);
    let smooth = sorted.iter().filter(|gap| **gap <= SMOOTH_MS).count();
    GapStats {
        count: sorted.len(),
        p50: percentile(&sorted, 0.5),
        p95: percentile(&sorted, 0.95),
        p99: percentile(&sorted, 0.99),
        max: sorted[sorted.len() - 1],
        smooth: smooth as f32 / sorted.len() as f32 * 100.0,
    }
}

/// The frame-gap figure the first benchmark (M1) reported: gaps between frame
/// ticks, ignoring ticks under 2 ms apart and gaps of 250 ms or more. It
/// cannot tell a slow frame from a pause between two animations, so a pause of
/// 20-250 ms after an animation settles (or a slower, intended frame rate)
/// counts as a slow frame. Kept to compare old and new reports.
pub(super) fn tick_gaps_m1(ticks: &[(Instant, Duration)]) -> Vec<f32> {
    let mut gaps = Vec::new();
    let mut last: Option<Instant> = None;
    for &(at, _) in ticks {
        if let Some(previous) = last {
            let gap = ms(at.saturating_duration_since(previous));
            if gap < 2.0 {
                continue;
            }
            if gap < 250.0 {
                gaps.push(gap);
            }
        }
        last = Some(at);
    }
    gaps.sort_by(f32::total_cmp);
    gaps
}

/// The screen's refresh interval, estimated from the fastest frames: most
/// frames come one refresh apart or a whole number of refreshes apart.
pub(super) fn refresh_estimate(gaps_ms: &[f32]) -> Option<f32> {
    if gaps_ms.len() < 20 {
        return None;
    }
    let mut sorted = gaps_ms.to_vec();
    sorted.sort_by(f32::total_cmp);
    Some(percentile(&sorted, 0.05)).filter(|gap| *gap > 1.0)
}

/// Share of gaps that took one refresh (the frame was ready in time).
pub(super) fn on_time_share(gaps_ms: &[f32], refresh_ms: f32) -> f32 {
    if gaps_ms.is_empty() || refresh_ms <= 0.0 {
        return 0.0;
    }
    let on_time = gaps_ms.iter().filter(|gap| **gap <= refresh_ms * 1.5).count();
    on_time as f32 / gaps_ms.len() as f32 * 100.0
}

/// How long until the result of an action was on its way to the screen: from
/// the action to the end of the renderer's work on the first frame after it.
#[derive(Clone, Copy, Debug)]
pub(super) struct Latency {
    pub(super) ms: f32,
    /// Part of `ms` before that frame started (the update, the rest of the
    /// UI, and waiting for the screen).
    pub(super) before_frame_ms: f32,
    pub(super) frame: RenderFrame,
}

pub(super) fn latency(at: Instant, frames: &[RenderFrame]) -> Option<Latency> {
    let frame = *frames.iter().find(|frame| frame.at >= at)?;
    Some(Latency {
        ms: ms(frame.at + frame.work.total() - at),
        before_frame_ms: ms(frame.at - at),
        frame,
    })
}

/// UI-thread CPU time used between two moments, from the nearest tick
/// samples (`ticks` is in time order; `start_cpu` is the reading at `from`).
/// Returns `(busy, wall)` in milliseconds.
pub(super) fn ui_busy(
    ticks: &[(Instant, Duration)],
    from: Instant,
    start_cpu: Duration,
    until: Instant,
) -> Option<(f32, f32)> {
    let &(at, cpu) = ticks.iter().find(|(at, _)| *at >= until)?;
    Some((ms(cpu.saturating_sub(start_cpu)), ms(at.saturating_duration_since(from))))
}

/// UI-thread work around a slow gap: from the last tick before it to the first after it.
pub(super) fn ui_busy_around(ticks: &[(Instant, Duration)], gap: &Gap) -> Option<(f32, f32)> {
    let &(from, start_cpu) = ticks.iter().rev().find(|(at, _)| *at <= gap.before.at)?;
    ui_busy(ticks, from, start_cpu, gap.frame.at)
}

pub(super) fn latency_stats(latencies: &[Latency]) -> (f32, f32, f32) {
    let mut sorted: Vec<f32> = latencies.iter().map(|latency| latency.ms).collect();
    sorted.sort_by(f32::total_cmp);
    (percentile(&sorted, 0.5), percentile(&sorted, 0.95), sorted.last().copied().unwrap_or(0.0))
}

pub(super) fn work_text(work: &RenderWork) -> String {
    let mut text = format!("renderer {:.1} ms", ms(work.total()));
    let mut parts = Vec::new();
    for (label, duration) in [
        ("pipelines", work.pipelines),
        ("render targets", work.targets),
        ("atlas growth", work.atlas_grow),
    ] {
        if duration >= Duration::from_micros(100) {
            parts.push(format!("{label} {:.1}", ms(duration)));
        }
    }
    if work.upload_count > 0 {
        parts.push(format!(
            "{} uploads {:.1} MB {:.1}",
            work.upload_count,
            work.upload_bytes as f64 / 1_048_576.0,
            ms(work.uploads)
        ));
    }
    if !parts.is_empty() {
        text.push_str(" (");
        text.push_str(&parts.join(", "));
        text.push(')');
    }
    text
}

pub(super) fn work_json(work: &RenderWork) -> Value {
    let ms2 = |duration: Duration| round2(duration.as_secs_f64() * 1000.0);
    json!({
        "total_ms": ms2(work.total()),
        "prepare_ms": ms2(work.prepare),
        "encode_ms": ms2(work.encode),
        "pipelines_ms": ms2(work.pipelines),
        "targets_ms": ms2(work.targets),
        "atlas_grow_ms": ms2(work.atlas_grow),
        "uploads_ms": ms2(work.uploads),
        "uploads": work.upload_count,
        "upload_bytes": work.upload_bytes,
    })
}

/// One layout switch: how long the first frame of the new layout took, and
/// the slowest frame until the next switch.
#[derive(Clone, Copy, Debug)]
pub(super) struct SwitchResult {
    pub(super) label: &'static str,
    pub(super) first: Option<Latency>,
    /// UI-thread CPU time and wall time from the switch to the tick after its first frame.
    pub(super) first_busy: Option<(f32, f32)>,
    pub(super) frames: usize,
    pub(super) slowest: Option<Gap>,
    pub(super) slowest_busy: Option<(f32, f32)>,
    /// How long the switch animated before the picker went idle.
    pub(super) settled_after_ms: Option<f32>,
    /// Median frame gap while the switch animated.
    pub(super) median_gap_ms: Option<f32>,
    /// Gap between the switch and the start of the slowest frame.
    pub(super) slowest_at_ms: Option<f32>,
}

pub(super) fn switch_results(
    events: &[BenchEvent],
    frames: &[RenderFrame],
    pauses: &[(Instant, Instant)],
    ticks: &[(Instant, Duration)],
    end: Instant,
) -> Vec<SwitchResult> {
    events
        .iter()
        .enumerate()
        .map(|(index, event)| {
            let until = events.get(index + 1).map_or(end, |next| next.at);
            let window: Vec<RenderFrame> = frames
                .iter()
                .filter(|frame| frame.at >= event.at && frame.at < until)
                .copied()
                .collect();
            let first = latency(event.at, &window);
            let first_busy = first.and_then(|first| {
                ui_busy(ticks, event.at, event.cpu, first.frame.at + first.frame.work.total())
            });
            let gaps = animated_gaps(&window, pauses);
            let median_gap_ms = (!gaps.is_empty()).then(|| gap_stats(&gaps).p50);
            let slowest = gaps.into_iter().max_by(|lhs, rhs| lhs.ms.total_cmp(&rhs.ms));
            let slowest_at_ms =
                slowest.map(|gap| ms(gap.before.at.saturating_duration_since(event.at)));
            let slowest_busy = slowest.as_ref().and_then(|gap| ui_busy_around(ticks, gap));
            let settled_after_ms = pauses
                .iter()
                .find(|(start, _)| *start > event.at && *start < until)
                .map(|(start, _)| ms(*start - event.at));
            SwitchResult {
                label: event.label,
                first,
                first_busy,
                frames: window.len(),
                slowest,
                slowest_busy,
                settled_after_ms,
                median_gap_ms,
                slowest_at_ms,
            }
        })
        .collect()
}

pub(super) fn busy_text(busy: Option<(f32, f32)>) -> String {
    busy.map_or_else(String::new, |(busy, wall)| {
        format!(", UI thread busy {busy:.0} of {wall:.0} ms")
    })
}

pub(super) fn switch_line(result: &SwitchResult) -> String {
    let first = result.first.map_or_else(
        || String::from("no frame"),
        |first| {
            format!(
                "{:>6.1} ms (waited {:.1} ms for the frame to start; {}{})",
                first.ms,
                first.before_frame_ms,
                work_text(&first.frame.work),
                busy_text(result.first_busy)
            )
        },
    );
    let slowest = result.slowest.map_or_else(String::new, |gap| {
        format!(
            "\n        frames {} (median {:.1} ms); slowest {:.1} ms, {:.0} ms after the switch ({}{})",
            result.frames,
            result.median_gap_ms.unwrap_or(0.0),
            gap.ms,
            result.slowest_at_ms.unwrap_or(0.0),
            work_text(&gap.frame.work),
            busy_text(result.slowest_busy)
        )
    });
    let settled = result.settled_after_ms.map_or_else(
        || String::from("; still moving at the next switch"),
        |settled| format!("; idle after {settled:.0} ms"),
    );
    format!("\n      {:<11} first frame {first}{slowest}{settled}", result.label)
}

pub(super) fn switch_json(result: &SwitchResult) -> Value {
    json!({
        "layout": result.label,
        "frames": result.frames,
        "first_frame_ms": result.first.map(|first| round2(f64::from(first.ms))),
        "first_frame_wait_ms": result.first.map(|first| round2(f64::from(first.before_frame_ms))),
        "first_frame_work": result.first.map(|first| work_json(&first.frame.work)),
        "first_frame_ui_busy_ms": result.first_busy.map(|(busy, _)| round2(f64::from(busy))),
        "slowest_gap_ms": result.slowest.map(|gap| round2(f64::from(gap.ms))),
        "slowest_gap_work": result.slowest.map(|gap| work_json(&gap.frame.work)),
        "slowest_gap_ui_busy_ms": result.slowest_busy.map(|(busy, _)| round2(f64::from(busy))),
        "settled_after_ms": result.settled_after_ms.map(|settled| round2(f64::from(settled))),
        "median_gap_ms": result.median_gap_ms.map(|gap| round2(f64::from(gap))),
        "slowest_at_ms": result.slowest_at_ms.map(|at| round2(f64::from(at))),
    })
}

#[cfg(test)]
mod tests;
