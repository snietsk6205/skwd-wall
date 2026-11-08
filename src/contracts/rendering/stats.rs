//! Render-side frame log. The scene renderer adds one record per drawn frame
//! (how long it waited since the last frame and what it did on the GPU side:
//! thumbnail uploads, pipeline and render-target creation, atlas growth); the
//! benchmark and the performance overlay read it. It is shared through the
//! scene primitive, so there is no global.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

/// Frames kept (about 14 s at 144 Hz).
pub const RENDER_LOG_FRAMES: usize = 2048;

/// What the scene renderer did while preparing one frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderWork {
    /// The whole `prepare` step.
    pub prepare: Duration,
    /// Copying thumbnail tiles to the GPU.
    pub uploads: Duration,
    pub upload_count: u32,
    pub upload_bytes: u64,
    /// Creating shaders and pipelines (only on their first use).
    pub pipelines: Duration,
    /// Creating offscreen render targets.
    pub targets: Duration,
    /// Growing the thumbnail texture arrays.
    pub atlas_grow: Duration,
    /// Recording the draw commands (`render`).
    pub encode: Duration,
}

impl RenderWork {
    /// Everything the scene renderer measured itself.
    pub fn total(&self) -> Duration {
        self.prepare + self.encode
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderFrame {
    /// When the renderer started preparing this frame.
    pub at: Instant,
    /// Time since the previous frame started (none for the first frame).
    pub gap: Option<Duration>,
    pub work: RenderWork,
}

#[derive(Debug, Default)]
struct Log {
    frames: VecDeque<RenderFrame>,
}

/// Cheap to clone; every clone writes to and reads from the same log.
#[derive(Clone, Debug, Default)]
pub struct RenderStats(Arc<Mutex<Log>>);

impl RenderStats {
    /// Adds a frame whose `prepare` started at `at`.
    pub fn record_prepare(&self, at: Instant, work: RenderWork) {
        let mut log = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let gap = log.frames.back().map(|last| at.saturating_duration_since(last.at));
        if log.frames.len() == RENDER_LOG_FRAMES {
            log.frames.pop_front();
        }
        log.frames.push_back(RenderFrame { at, gap, work });
    }

    /// Adds the draw-command recording time to the newest frame.
    pub fn record_encode(&self, encode: Duration) {
        let mut log = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(last) = log.frames.back_mut() {
            last.work.encode += encode;
        }
    }

    /// Frames that started at or after `from` and before `to`.
    pub fn frames_between(&self, from: Instant, to: Instant) -> Vec<RenderFrame> {
        let log = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        log.frames.iter().filter(|frame| frame.at >= from && frame.at < to).copied().collect()
    }

    /// The newest `count` frames, oldest first.
    pub fn recent(&self, count: usize) -> Vec<RenderFrame> {
        let log = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        log.frames.iter().skip(log.frames.len().saturating_sub(count)).copied().collect()
    }
}

#[cfg(test)]
mod tests;
