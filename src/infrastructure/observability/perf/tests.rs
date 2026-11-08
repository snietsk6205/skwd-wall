#![cfg(test)]

use std::time::Duration;

use super::*;

#[test]
fn records_count_mean_and_max() {
    record("perf-test-a", Duration::from_micros(1000));
    record("perf-test-a", Duration::from_micros(3000));
    let stat = stat("perf-test-a");
    assert!(stat.count >= 2);
    assert!(stat.max_us >= 3000);
    assert!(stat.mean_ms() > 0.0);
}

#[test]
fn span_guard_records_on_drop() {
    {
        let _span = span("perf-test-span");
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(stat("perf-test-span").last_us >= 1000);
}

#[test]
fn cpu_percent_handles_zero_wall_time() {
    assert_eq!(cpu_percent(Duration::ZERO, Duration::from_secs(1), Duration::ZERO), 0.0);
    let half = cpu_percent(Duration::ZERO, Duration::from_millis(500), Duration::from_secs(1));
    assert!((half - 50.0).abs() < 1e-9);
}

#[test]
fn cpu_time_is_monotonic() {
    let before = cpu_time();
    let mut spin = 0u64;
    for value in 0..200_000u64 {
        spin = spin.wrapping_add(value * value);
    }
    assert!(spin > 0);
    assert!(cpu_time() >= before);
}

#[test]
fn write_report_saves_latest_and_dated_copies() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = write_report(&dir.path().display().to_string(), "hello", "{}").expect("written");
    assert!(path.ends_with("latest.txt"));
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "hello\n");
    let dated = std::fs::read_dir(dir.path().join("bench"))
        .expect("bench dir")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("bench-"))
        .count();
    assert_eq!(dated, 2);
}

#[test]
fn thread_cpu_time_counts_work_not_sleep() {
    let before = thread_cpu_time();
    std::thread::sleep(Duration::from_millis(20));
    let slept = thread_cpu_time().saturating_sub(before);
    assert!(slept < Duration::from_millis(15), "sleeping used {slept:?} of CPU");
    let started = std::time::Instant::now();
    let mut spin = 0_u64;
    while started.elapsed() < Duration::from_millis(20) {
        spin = std::hint::black_box(spin.wrapping_add(1));
    }
    assert!(thread_cpu_time().saturating_sub(before) >= Duration::from_millis(10));
}
