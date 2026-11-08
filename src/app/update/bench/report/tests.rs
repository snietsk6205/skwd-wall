use std::time::{Duration, Instant};

use super::{
    animated_gaps, gap_stats, latency, on_time_share, refresh_estimate, switch_results,
    tick_gaps_m1, ui_busy, work_text,
};
use crate::app::state::BenchEvent;
use crate::contracts::rendering::{RenderFrame, RenderWork};

fn frame(start: Instant, at_ms: u64, prepare_ms: u64) -> RenderFrame {
    RenderFrame {
        at: start + Duration::from_millis(at_ms),
        gap: None,
        work: RenderWork { prepare: Duration::from_millis(prepare_ms), ..RenderWork::default() },
    }
}

fn at(start: Instant, ms: u64) -> Instant {
    start + Duration::from_millis(ms)
}

#[test]
fn gaps_across_pauses_are_not_slow_frames() {
    let start = Instant::now();
    // Frames every 7 ms, then the picker settles (pause 30..230 ms), then frames again.
    let frames: Vec<_> =
        [0, 7, 14, 21, 230, 237, 244].iter().map(|ms| frame(start, *ms, 1)).collect();
    let pauses = [(at(start, 25), at(start, 228))];
    let gaps = animated_gaps(&frames, &pauses);
    assert_eq!(gaps.len(), 5);
    assert!(gaps.iter().all(|gap| gap.ms < 8.0));
    let stats = gap_stats(&gaps);
    assert!((stats.max - 7.0).abs() < 0.01);
    assert!((stats.smooth - 100.0).abs() < 0.01);
    // Without the pause the 209 ms wait would count as a slow frame.
    assert!((gap_stats(&animated_gaps(&frames, &[])).max - 209.0).abs() < 0.01);
}

#[test]
fn first_benchmark_counted_pauses_as_frames() {
    let start = Instant::now();
    let tick = |ms: u64| (at(start, ms), Duration::ZERO);
    // Animation ticks, a 200 ms pause, a tick 1 ms after another (same frame), a 300 ms pause.
    let ticks = [tick(0), tick(7), tick(14), tick(214), tick(215), tick(515)];
    let gaps = tick_gaps_m1(&ticks);
    assert_eq!(gaps.len(), 3);
    assert!((gaps[2] - 200.0).abs() < 0.01);
}

#[test]
fn latency_runs_to_the_end_of_the_next_frame() {
    let start = Instant::now();
    let frames = [frame(start, 0, 1), frame(start, 12, 2), frame(start, 19, 2)];
    let measured = latency(at(start, 5), &frames).unwrap();
    assert!((measured.ms - 9.0).abs() < 0.01);
    assert!((measured.before_frame_ms - 7.0).abs() < 0.01);
    assert!(latency(at(start, 50), &frames).is_none());
}

#[test]
fn refresh_from_the_fastest_frames() {
    let mut gaps = vec![6.9_f32; 40];
    gaps.extend([13.9_f32; 60]);
    let refresh = refresh_estimate(&gaps).unwrap();
    assert!((refresh - 6.9).abs() < 0.01);
    assert!((on_time_share(&gaps, refresh) - 40.0).abs() < 0.01);
    assert!(refresh_estimate(&gaps[..10]).is_none());
}

#[test]
fn ui_busy_uses_the_tick_after() {
    let start = Instant::now();
    let ticks = [
        (at(start, 0), Duration::from_millis(100)),
        (at(start, 30), Duration::from_millis(104)),
        (at(start, 60), Duration::from_millis(130)),
    ];
    let (busy, wall) = ui_busy(&ticks, start, Duration::from_millis(100), at(start, 20)).unwrap();
    assert!((busy - 4.0).abs() < 0.01);
    assert!((wall - 30.0).abs() < 0.01);
    assert!(ui_busy(&ticks, start, Duration::ZERO, at(start, 61)).is_none());
}

#[test]
fn switch_results_per_layout() {
    let start = Instant::now();
    let events = [
        BenchEvent { at: at(start, 0), label: "hex", cpu: Duration::ZERO },
        BenchEvent { at: at(start, 100), label: "hand", cpu: Duration::ZERO },
    ];
    let frames: Vec<_> =
        [2, 9, 40, 47, 102, 109, 116].iter().map(|ms| frame(start, *ms, 1)).collect();
    let pauses = [(at(start, 50), at(start, 100))];
    let ticks: Vec<_> = (0..14).map(|step| (at(start, step * 10), Duration::ZERO)).collect();
    let results = switch_results(&events, &frames, &pauses, &ticks, at(start, 140));
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].frames, 4);
    // The 31 ms gap at 9..40 is a slow frame while animating; idle came at 50 ms.
    assert!((results[0].slowest.unwrap().ms - 31.0).abs() < 0.01);
    assert!((results[0].settled_after_ms.unwrap() - 50.0).abs() < 0.01);
    assert!((results[1].first.unwrap().ms - 3.0).abs() < 0.01);
    assert!(results[1].settled_after_ms.is_none());
}

#[test]
fn work_text_lists_only_what_happened() {
    let quiet = RenderWork { prepare: Duration::from_millis(1), ..RenderWork::default() };
    assert_eq!(work_text(&quiet), "renderer 1.0 ms");
    let busy = RenderWork {
        prepare: Duration::from_millis(9),
        pipelines: Duration::from_millis(5),
        upload_count: 2,
        upload_bytes: 2 * 1_048_576,
        uploads: Duration::from_millis(1),
        ..RenderWork::default()
    };
    assert_eq!(work_text(&busy), "renderer 9.0 ms (pipelines 5.0, 2 uploads 2.0 MB 1.0)");
}
