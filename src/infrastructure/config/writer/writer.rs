//! Writes `config.json` on a background thread (custom build).
//!
//! Saving used to happen on the UI thread: serialise, write, then two
//! `fsync` calls (file and folder). On btrfs and on busy disks an `fsync` can
//! take tens of milliseconds, so every settings click stalled the picker.
//!
//! Now a save hands the finished text to this writer and returns at once.
//! Saves that arrive within [`DEBOUNCE`] of each other are merged into one
//! write (and so into one config reload in the daemon). [`flush`] waits until
//! everything is on disk; it runs before every daemon request, so the daemon
//! never acts on an older config than the picker has.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Saves closer together than this are merged into one write.
pub const DEBOUNCE: Duration = Duration::from_millis(120);
/// A burst of saves is written at the latest after this long.
pub const MAX_DELAY: Duration = Duration::from_millis(600);

#[derive(Default)]
struct State {
    pending: Vec<(PathBuf, Vec<u8>)>,
    first_at: Option<Instant>,
    last_at: Option<Instant>,
    urgent: bool,
    busy: bool,
    written: u64,
}

pub struct Writer {
    state: Mutex<State>,
    wake: Condvar,
}

impl Writer {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { state: Mutex::new(State::default()), wake: Condvar::new() })
    }

    /// Starts the background thread for this writer.
    pub fn spawn(self: &Arc<Self>) -> bool {
        let writer = Arc::clone(self);
        std::thread::Builder::new().name("config-writer".into()).spawn(move || writer.run()).is_ok()
    }

    /// Queues `bytes` for `path`, replacing anything still queued for it.
    pub fn enqueue(&self, path: &Path, bytes: Vec<u8>) {
        let Ok(mut state) = self.state.lock() else { return };
        let now = Instant::now();
        if let Some(slot) = state.pending.iter_mut().find(|(queued, _)| queued == path) {
            slot.1 = bytes;
        } else {
            state.pending.push((path.to_path_buf(), bytes));
        }
        state.first_at.get_or_insert(now);
        state.last_at = Some(now);
        self.wake.notify_all();
    }

    /// Whether a write for `path` is queued or in progress.
    pub fn is_pending(&self, path: &Path) -> bool {
        self.state
            .lock()
            .is_ok_and(|state| state.busy || state.pending.iter().any(|(queued, _)| queued == path))
    }

    /// Blocks until every queued write has reached the disk (at most `limit`).
    pub fn flush(&self, limit: Duration) -> bool {
        let Ok(mut state) = self.state.lock() else { return false };
        if state.pending.is_empty() && !state.busy {
            return true;
        }
        state.urgent = true;
        self.wake.notify_all();
        let deadline = Instant::now() + limit;
        while !state.pending.is_empty() || state.busy {
            let now = Instant::now();
            if now >= deadline {
                return false;
            }
            let Ok((next, _)) = self.wake.wait_timeout(state, deadline - now) else {
                return false;
            };
            state = next;
        }
        true
    }

    /// Number of writes done so far.
    pub fn written(&self) -> u64 {
        self.state.lock().map_or(0, |state| state.written)
    }

    fn run(&self) {
        loop {
            let Ok(mut state) = self.state.lock() else { return };
            while state.pending.is_empty() {
                let Ok(next) = self.wake.wait(state) else { return };
                state = next;
            }
            loop {
                if state.urgent {
                    break;
                }
                let now = Instant::now();
                let first = state.first_at.unwrap_or(now);
                let last = state.last_at.unwrap_or(now);
                let due = (last + DEBOUNCE).min(first + MAX_DELAY);
                if now >= due {
                    break;
                }
                let Ok((next, _)) = self.wake.wait_timeout(state, due - now) else { return };
                state = next;
            }
            let batch = std::mem::take(&mut state.pending);
            state.first_at = None;
            state.last_at = None;
            state.urgent = false;
            state.busy = true;
            drop(state);
            for (path, bytes) in &batch {
                write_now(path, bytes);
            }
            let Ok(mut state) = self.state.lock() else { return };
            state.busy = false;
            state.written += batch.len() as u64;
            drop(state);
            self.wake.notify_all();
        }
    }
}

fn write_now(path: &Path, bytes: &[u8]) {
    let _span = crate::infrastructure::observability::perf::span("config.write_disk");
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    match skwd_config::atomic_write_mode(path, bytes, Some(0o600)) {
        Ok(()) => log::info!("config saved to {}", path.display()),
        Err(err) => log::warn!("config write failed: {err}"),
    }
}

static GLOBAL: OnceLock<Option<Arc<Writer>>> = OnceLock::new();

fn global() -> Option<&'static Arc<Writer>> {
    GLOBAL
        .get_or_init(|| {
            let writer = Writer::new();
            writer.spawn().then_some(writer)
        })
        .as_ref()
}

/// `SKWD_WALL_SYNC_CONFIG=1` brings back the old behaviour (write and fsync
/// on the UI thread) so the two can be compared with `skwd-helm ui bench`.
fn synchronous() -> bool {
    static SYNC: OnceLock<bool> = OnceLock::new();
    *SYNC.get_or_init(|| {
        std::env::var("SKWD_WALL_SYNC_CONFIG").is_ok_and(|value| value == "1" || value == "true")
    })
}

/// Saves `bytes` to `path` soon, off the UI thread. Unit tests write at once.
pub fn save(path: &Path, bytes: Vec<u8>) {
    if cfg!(test) || synchronous() {
        write_now(path, &bytes);
        return;
    }
    match global() {
        Some(writer) => writer.enqueue(path, bytes),
        None => write_now(path, &bytes),
    }
}

/// Waits until every queued config write is on disk.
pub fn flush() {
    if let Some(Some(writer)) = GLOBAL.get() {
        let _span = crate::infrastructure::observability::perf::span("config.flush_wait");
        if !writer.flush(Duration::from_secs(3)) {
            log::warn!("config writer: flush timed out");
        }
    }
}

/// Whether a write to `path` is still on its way to the disk.
pub fn pending(path: &Path) -> bool {
    matches!(GLOBAL.get(), Some(Some(writer)) if writer.is_pending(path))
}

/// How many config writes reached the disk since the picker started.
pub fn writes() -> u64 {
    match GLOBAL.get() {
        Some(Some(writer)) => writer.written(),
        _ => 0,
    }
}
