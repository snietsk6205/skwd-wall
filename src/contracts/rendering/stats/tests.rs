use std::time::{Duration, Instant};

use super::{RENDER_LOG_FRAMES, RenderStats, RenderWork};

fn work(prepare_ms: u64) -> RenderWork {
    RenderWork { prepare: Duration::from_millis(prepare_ms), ..RenderWork::default() }
}

#[test]
fn gaps_between_frames() {
    let stats = RenderStats::default();
    let start = Instant::now();
    stats.record_prepare(start, work(1));
    stats.record_prepare(start + Duration::from_millis(7), work(2));
    stats.record_encode(Duration::from_millis(3));
    let frames = stats.frames_between(start, start + Duration::from_secs(1));
    assert_eq!(frames.len(), 2);
    assert_eq!(frames[0].gap, None);
    assert_eq!(frames[1].gap, Some(Duration::from_millis(7)));
    assert_eq!(frames[1].work.encode, Duration::from_millis(3));
    assert_eq!(frames[1].work.total(), Duration::from_millis(5));
}

#[test]
fn window_and_recent() {
    let stats = RenderStats::default();
    let start = Instant::now();
    for index in 0..5 {
        stats.record_prepare(start + Duration::from_millis(10 * index), work(index));
    }
    let middle =
        stats.frames_between(start + Duration::from_millis(10), start + Duration::from_millis(30));
    assert_eq!(middle.len(), 2);
    assert_eq!(middle[0].at, start + Duration::from_millis(10));
    assert!(
        stats
            .frames_between(start + Duration::from_secs(1), start + Duration::from_secs(2))
            .is_empty()
    );
    assert_eq!(stats.recent(2).len(), 2);
    assert_eq!(stats.recent(2)[1].at, start + Duration::from_millis(40));
}

#[test]
fn log_is_bounded() {
    let stats = RenderStats::default();
    let start = Instant::now();
    for index in 0..(RENDER_LOG_FRAMES as u64 + 10) {
        stats.record_prepare(start + Duration::from_micros(index), RenderWork::default());
    }
    assert_eq!(stats.recent(usize::MAX).len(), RENDER_LOG_FRAMES);
}
