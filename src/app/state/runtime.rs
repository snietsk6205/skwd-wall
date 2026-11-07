use std::time::{Duration, Instant};

use futures_channel::mpsc::UnboundedSender;

use crate::infrastructure::observability::metrics::Metrics;
use crate::infrastructure::runtime::{FrameClock, Wake};

use super::{DemoSession, PerfState};

pub(crate) struct AppRuntimeState {
    pub(crate) picker_output: Option<String>,
    pub(crate) overlay: Option<iced::window::Id>,
    pub(crate) stashed: bool,
    pub(crate) toast: Option<(String, Instant)>,
    pub(crate) metrics: Metrics,
    pub(crate) perf: PerfState,
    pub(crate) last_animation_tick: Option<Instant>,
    pub(crate) last_tick: Option<Instant>,
    pub(crate) next_animation_tick: Option<Instant>,
    pub(crate) animation_interval: Option<Duration>,
    pub(crate) demo: Option<DemoSession>,
    pub(crate) semantic: Option<crate::infrastructure::semantic::SemanticService>,
    pub(crate) frame_clock: FrameClock,
    pub(crate) wake_tx: UnboundedSender<Wake>,
}

impl AppRuntimeState {
    pub(crate) fn new(frame_clock: FrameClock, wake_tx: UnboundedSender<Wake>) -> Self {
        Self {
            picker_output: None,
            overlay: None,
            stashed: false,
            toast: None,
            metrics: Metrics::new(),
            perf: PerfState::default(),
            last_animation_tick: None,
            last_tick: None,
            next_animation_tick: None,
            animation_interval: None,
            demo: None,
            semantic: None,
            frame_clock,
            wake_tx,
        }
    }
}
