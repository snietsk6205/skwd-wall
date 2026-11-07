use std::time::{Duration, Instant};

use crate::infrastructure::observability::perf::CpuMeter;

const GAPS: usize = 120;
/// Ticks closer together than this belong to the same frame (no screen runs at 500 Hz).
const SAME_FRAME_MS: f32 = 2.0;

/// Performance overlay and benchmark state.
#[derive(Default)]
pub(crate) struct PerfState {
    pub(crate) cpu: CpuMeter,
    pub(crate) bench: Option<BenchRun>,
    pub(crate) last_report: Option<String>,
    pub(crate) last_report_path: Option<String>,
    last_frame: Option<Instant>,
    gaps_ms: std::collections::VecDeque<f32>,
}

impl PerfState {
    /// Called once per drawn frame.
    pub(crate) fn on_frame(&mut self, now: Instant) {
        if let Some(last) = self.last_frame {
            let gap = now.saturating_duration_since(last).as_secs_f32() * 1000.0;
            // Extra ticks inside the same frame (a message arriving right
            // after the frame tick) are not new frames.
            if gap < SAME_FRAME_MS {
                return;
            }
            // Gaps longer than this are pauses between animations, not frames.
            if gap < 100.0 {
                if self.gaps_ms.len() == GAPS {
                    self.gaps_ms.pop_front();
                }
                self.gaps_ms.push_back(gap);
            }
        }
        self.last_frame = Some(now);
        if let Some(bench) = self.bench.as_mut() {
            bench.on_frame(now);
        }
        self.cpu.sample(now, Duration::from_millis(900));
    }

    /// Frames per second and the 50th/95th percentile frame gap of recent animation.
    pub(crate) fn frame_stats(&self) -> (f32, f32, f32) {
        if self.gaps_ms.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        let mut sorted: Vec<f32> = self.gaps_ms.iter().copied().collect();
        sorted.sort_by(f32::total_cmp);
        let pick = |fraction: f32| sorted[((sorted.len() - 1) as f32 * fraction).round() as usize];
        let p50 = pick(0.5);
        let fps = if p50 > 0.0 { 1000.0 / p50 } else { 0.0 };
        (fps, p50, pick(0.95))
    }
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
}

pub(crate) struct BenchPhase {
    pub(crate) name: &'static str,
    pub(crate) started: Instant,
    pub(crate) cpu_at_start: Duration,
    pub(crate) views_at_start: u64,
    pub(crate) last_frame: Option<Instant>,
    pub(crate) frame_gaps_ms: Vec<f32>,
}

impl BenchRun {
    pub(crate) fn on_frame(&mut self, now: Instant) {
        let Some(phase) = self.phase.as_mut() else { return };
        if let Some(last) = phase.last_frame {
            let gap = now.saturating_duration_since(last).as_secs_f32() * 1000.0;
            if gap < SAME_FRAME_MS {
                return;
            }
            if gap < 250.0 {
                phase.frame_gaps_ms.push(gap);
            }
        }
        phase.last_frame = Some(now);
    }
}
