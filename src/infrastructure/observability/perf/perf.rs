//! Always-on, very cheap timing of the picker's coarse steps (a settings
//! change, a layout rebuild, a config save, a view build, a frame tick).
//!
//! Each measured step costs two clock reads and one uncontended lock. The
//! numbers feed the performance overlay, `skwd-helm ui perf` and
//! `skwd-helm ui bench`; steps slower than [`SLOW_MS`] are also written to the
//! log at debug level (`SKWD_WALL_LOG=debug`).

use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Steps slower than this are logged at debug level.
pub const SLOW_MS: f64 = 4.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpanStat {
    pub count: u64,
    pub total_us: u64,
    pub max_us: u64,
    pub last_us: u64,
}

impl SpanStat {
    pub fn mean_ms(&self) -> f64 {
        if self.count == 0 { 0.0 } else { self.total_us as f64 / self.count as f64 / 1000.0 }
    }

    pub fn max_ms(&self) -> f64 {
        self.max_us as f64 / 1000.0
    }

    pub fn last_ms(&self) -> f64 {
        self.last_us as f64 / 1000.0
    }

    fn add(&mut self, micros: u64) {
        self.count += 1;
        self.total_us = self.total_us.saturating_add(micros);
        self.max_us = self.max_us.max(micros);
        self.last_us = micros;
    }
}

struct Table {
    names: Vec<&'static str>,
    stats: Vec<SpanStat>,
}

impl Table {
    const fn new() -> Self {
        Self { names: Vec::new(), stats: Vec::new() }
    }

    fn add(&mut self, name: &'static str, micros: u64) {
        let index = if let Some(index) = self.names.iter().position(|known| *known == name) {
            index
        } else {
            self.names.push(name);
            self.stats.push(SpanStat::default());
            self.names.len() - 1
        };
        self.stats[index].add(micros);
    }
}

static TABLE: Mutex<Table> = Mutex::new(Table::new());

/// Measures from creation until it is dropped.
#[must_use = "the step is measured until this guard is dropped"]
pub struct Span {
    name: &'static str,
    start: Instant,
}

impl Drop for Span {
    fn drop(&mut self) {
        record(self.name, self.start.elapsed());
    }
}

pub fn span(name: &'static str) -> Span {
    Span { name, start: Instant::now() }
}

pub fn record(name: &'static str, duration: Duration) {
    let micros = duration.as_micros().min(u128::from(u64::MAX)) as u64;
    let ms = micros as f64 / 1000.0;
    if ms >= SLOW_MS {
        log::debug!("perf: {name} took {ms:.2} ms");
    }
    if let Ok(mut table) = TABLE.lock() {
        table.add(name, micros);
    }
}

/// Every step measured so far, slowest average first.
pub fn snapshot() -> Vec<(&'static str, SpanStat)> {
    let Ok(table) = TABLE.lock() else { return Vec::new() };
    let mut rows: Vec<_> = table.names.iter().copied().zip(table.stats.iter().copied()).collect();
    rows.sort_by(|lhs, rhs| rhs.1.mean_ms().total_cmp(&lhs.1.mean_ms()));
    rows
}

pub fn stat(name: &str) -> SpanStat {
    let Ok(table) = TABLE.lock() else { return SpanStat::default() };
    table
        .names
        .iter()
        .position(|known| *known == name)
        .map_or_else(SpanStat::default, |index| table.stats[index])
}

pub fn reset() {
    if let Ok(mut table) = TABLE.lock() {
        *table = Table::new();
    }
}

/// CPU time (user + system) this process has used since it started.
pub fn cpu_time() -> Duration {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::zeroed();
    // SAFETY: `usage` is a valid, writable rusage struct for the duration of the call.
    let ok = unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) } == 0;
    if !ok {
        return Duration::ZERO;
    }
    // SAFETY: getrusage succeeded and filled the struct.
    let usage = unsafe { usage.assume_init() };
    let micros = |time: libc::timeval| {
        Duration::from_secs(time.tv_sec.max(0) as u64)
            + Duration::from_micros(time.tv_usec.max(0) as u64)
    };
    micros(usage.ru_utime) + micros(usage.ru_stime)
}

/// CPU time (user + system) the calling thread has used. On the UI thread this
/// tells working apart from waiting (for the screen, the GPU or the driver).
pub fn thread_cpu_time() -> Duration {
    let mut time = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: `time` is a valid, writable timespec for the duration of the call.
    let ok = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &raw mut time) } == 0;
    if !ok {
        return Duration::ZERO;
    }
    Duration::from_secs(time.tv_sec.max(0) as u64)
        + Duration::from_nanos(time.tv_nsec.max(0) as u64)
}

/// Turns two CPU-time readings into a percentage of one core.
pub fn cpu_percent(cpu_before: Duration, cpu_after: Duration, wall: Duration) -> f64 {
    if wall.is_zero() {
        return 0.0;
    }
    cpu_after.saturating_sub(cpu_before).as_secs_f64() / wall.as_secs_f64() * 100.0
}

/// Samples CPU use over a sliding window for the performance overlay.
#[derive(Clone, Copy, Debug)]
pub struct CpuMeter {
    since: Instant,
    cpu_since: Duration,
    pub percent: f64,
}

impl CpuMeter {
    pub fn new() -> Self {
        Self { since: Instant::now(), cpu_since: cpu_time(), percent: 0.0 }
    }

    /// Updates `percent` once at least `every` has passed; returns whether it changed.
    pub fn sample(&mut self, now: Instant, every: Duration) -> bool {
        let wall = now.saturating_duration_since(self.since);
        if wall < every {
            return false;
        }
        let cpu = cpu_time();
        self.percent = cpu_percent(self.cpu_since, cpu, wall);
        self.since = now;
        self.cpu_since = cpu;
        true
    }
}

impl Default for CpuMeter {
    fn default() -> Self {
        Self::new()
    }
}

/// Saves a benchmark report as `bench/latest.txt` (+ `.json`) and a dated copy
/// in the cache folder. Returns the path of the text report.
pub fn write_report(cache_dir: &str, text: &str, json: &str) -> Option<String> {
    let dir = std::path::Path::new(cache_dir).join("bench");
    std::fs::create_dir_all(&dir).ok()?;
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let dated = dir.join(format!("bench-{stamp}.txt"));
    std::fs::write(&dated, format!("{text}\n")).ok()?;
    let _ = std::fs::write(dir.join(format!("bench-{stamp}.json")), json);
    let latest = dir.join("latest.txt");
    let _ = std::fs::write(&latest, format!("{text}\n"));
    let _ = std::fs::write(dir.join("latest.json"), json);
    Some(latest.display().to_string())
}
