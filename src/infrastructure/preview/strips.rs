//! R1b (custom build): short frame strips for the cards in Settings. Each
//! transition is rendered ONCE by `skwd-wall-vk --preview-stream` (a dozen
//! small frames, one after the other, at low priority), saved as one small
//! picture in the cache folder, and from then on only read from disk. A card
//! shows the middle frame; the card the pointer or keyboard is on gets the
//! whole strip. The cache is keyed by the renderer file, so a new renderer
//! renders them again.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};

use futures_channel::mpsc::UnboundedSender;

use crate::infrastructure::runtime::Wake;

pub(crate) const STRIP_W: u32 = 240;
pub(crate) const STRIP_H: u32 = 135;
/// Frames kept per strip.
pub(crate) const STRIP_FRAMES: usize = 12;
/// The frame a still card shows: a little before half-way, where most
/// transitions show their shape best (several are plain black or white at
/// exactly half-way).
pub(crate) const STRIP_MIDDLE: usize = 4;
/// Changes when the strip file format does.
const FORMAT: u32 = 1;
const FRAME_BYTES: usize = STRIP_W as usize * STRIP_H as usize * 4;

/// One strip to render: its cache name, the renderer's `--shader` value and
/// extra environment (the sketch styles use `SKWD_PAPER_SKETCH`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StripJob {
    pub(crate) name: String,
    pub(crate) shader: String,
    pub(crate) env: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub enum StripEvent {
    /// The middle frame of a strip (RGBA).
    Middle { name: String, pixels: Arc<Vec<u8>> },
    /// Every frame of one strip (RGBA), for the card that plays.
    Frames { name: String, frames: Arc<Vec<Vec<u8>>> },
    /// A strip was rendered and saved.
    Made { name: String },
    /// The renderer could not draw this one.
    Failed { name: String },
}

enum Load {
    Middles(Vec<String>),
    Frames(String),
}

/// File name of a strip in the cache folder.
pub(crate) fn strip_file_name(name: &str) -> String {
    let safe: String =
        name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect();
    format!("{safe}.jpg")
}

fn fnv(bytes: &[u8], mut hash: u64) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// Cache key: the renderer file (size and change time), the strip format
/// and the two stock pictures.
pub(crate) fn cache_key(renderer: (u64, u128), stock: (u64, u64)) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    hash = fnv(&renderer.0.to_le_bytes(), hash);
    hash = fnv(&renderer.1.to_le_bytes(), hash);
    hash = fnv(&FORMAT.to_le_bytes(), hash);
    hash = fnv(&STRIP_W.to_le_bytes(), hash);
    hash = fnv(&(STRIP_FRAMES as u32).to_le_bytes(), hash);
    hash = fnv(&stock.0.to_le_bytes(), hash);
    hash = fnv(&stock.1.to_le_bytes(), hash);
    format!("{hash:016x}")
}

fn renderer_path() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("skwd-wall-vk")))
        .filter(|path| path.exists())
        .unwrap_or_else(|| PathBuf::from("skwd-wall-vk"))
}

fn file_stamp(path: &Path) -> (u64, u128) {
    let Ok(meta) = std::fs::metadata(path) else {
        return (0, 0);
    };
    let changed = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |since| since.as_nanos());
    (meta.len(), changed)
}

/// Cuts the frames a strip keeps out of the renderer's output (it sends
/// the untouched old picture first and the finished new one last).
pub(crate) fn kept_frames(raw: &[u8]) -> Option<Vec<&[u8]>> {
    let frames: Vec<&[u8]> = raw.chunks_exact(FRAME_BYTES).collect();
    (frames.len() >= STRIP_FRAMES + 2).then(|| frames[1..=STRIP_FRAMES].to_vec())
}

/// The frames side by side as one JPEG.
pub(crate) fn encode_strip(frames: &[&[u8]]) -> Option<Vec<u8>> {
    let count = frames.len() as u32;
    let mut sheet = image::RgbImage::new(STRIP_W * count, STRIP_H);
    for (index, frame) in frames.iter().enumerate() {
        if frame.len() != FRAME_BYTES {
            return None;
        }
        let left = index as u32 * STRIP_W;
        for (pixel, rgba) in frame.chunks_exact(4).enumerate() {
            let (x, y) = (pixel as u32 % STRIP_W, pixel as u32 / STRIP_W);
            sheet.put_pixel(left + x, y, image::Rgb([rgba[0], rgba[1], rgba[2]]));
        }
    }
    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 88).encode_image(&sheet).ok()?;
    Some(out)
}

/// The frames of a saved strip (RGBA each).
pub(crate) fn decode_strip(bytes: &[u8]) -> Option<Vec<Vec<u8>>> {
    let sheet = image::load_from_memory(bytes).ok()?.to_rgba8();
    if sheet.height() != STRIP_H || sheet.width() != STRIP_W * STRIP_FRAMES as u32 {
        return None;
    }
    let raw = sheet.as_raw();
    let row = sheet.width() as usize * 4;
    let mut frames = vec![Vec::with_capacity(FRAME_BYTES); STRIP_FRAMES];
    for y in 0..STRIP_H as usize {
        let line = &raw[y * row..(y + 1) * row];
        for (index, frame) in frames.iter_mut().enumerate() {
            let left = index * STRIP_W as usize * 4;
            frame.extend_from_slice(&line[left..left + STRIP_W as usize * 4]);
        }
    }
    Some(frames)
}

struct Shared {
    dir: PathBuf,
    wake: UnboundedSender<Wake>,
    paused: AtomicBool,
    stopped: AtomicBool,
    /// The renderer that is drawing a strip right now (0 = none).
    child: AtomicU32,
    queue: Mutex<Vec<StripJob>>,
}

impl Shared {
    fn send(&self, event: StripEvent) {
        let _ = self.wake.unbounded_send(Wake::Strip(event));
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(strip_file_name(name))
    }
}

/// Owns the two background threads (reading strips, rendering strips).
pub(crate) struct StripWorker {
    shared: Arc<Shared>,
    load: Sender<Load>,
    make: Sender<()>,
}

impl StripWorker {
    /// `cache_dir` is the picker's cache folder; `stock` the two preview
    /// pictures (old, new).
    pub(crate) fn start(
        cache_dir: &str,
        stock: (String, String),
        wake: UnboundedSender<Wake>,
    ) -> Option<Self> {
        let renderer = renderer_path();
        let sizes = (file_stamp(Path::new(&stock.0)).0, file_stamp(Path::new(&stock.1)).0);
        let root = Path::new(cache_dir).join("settings-strips");
        let key = cache_key(file_stamp(&renderer), sizes);
        let dir = root.join(&key);
        std::fs::create_dir_all(&dir).ok()?;
        // Strips of other renderer versions are of no use any more.
        if let Ok(entries) = std::fs::read_dir(&root) {
            for entry in entries.flatten() {
                if entry.file_name().to_string_lossy() != key {
                    let _ = std::fs::remove_dir_all(entry.path());
                }
            }
        }
        let shared = Arc::new(Shared {
            dir,
            wake,
            paused: AtomicBool::new(false),
            stopped: AtomicBool::new(false),
            child: AtomicU32::new(0),
            queue: Mutex::new(Vec::new()),
        });
        let (load, load_rx) = channel::<Load>();
        let (make, make_rx) = channel::<()>();
        let for_load = Arc::clone(&shared);
        std::thread::Builder::new()
            .name("strip-load".into())
            .spawn(move || load_loop(&for_load, &load_rx))
            .ok()?;
        let for_make = Arc::clone(&shared);
        std::thread::Builder::new()
            .name("strip-make".into())
            .spawn(move || make_loop(&for_make, &make_rx, &renderer, &stock))
            .ok()?;
        Some(Self { shared, load, make })
    }

    /// Whether a strip is saved already.
    pub(crate) fn cached(&self, name: &str) -> bool {
        self.shared.path(name).is_file()
    }

    /// Reads the middle frames of these (saved) strips.
    pub(crate) fn load_middles(&self, names: Vec<String>) {
        if !names.is_empty() {
            let _ = self.load.send(Load::Middles(names));
        }
    }

    /// Reads every frame of one (saved) strip.
    pub(crate) fn load_frames(&self, name: &str) {
        let _ = self.load.send(Load::Frames(name.to_string()));
    }

    /// Replaces the list of strips still to render (first = next).
    pub(crate) fn set_queue(&self, jobs: Vec<StripJob>) {
        *self.shared.queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = jobs;
        let _ = self.make.send(());
    }

    /// Pauses rendering (benchmark, Low-power mode, Settings hidden): the
    /// renderer at work is stopped at once, the rest waits.
    pub(crate) fn set_paused(&self, paused: bool) {
        if self.shared.paused.swap(paused, Ordering::SeqCst) == paused {
            return;
        }
        if paused {
            kill_child(&self.shared);
        } else {
            let _ = self.make.send(());
        }
    }

    pub(crate) fn paused(&self) -> bool {
        self.shared.paused.load(Ordering::SeqCst)
    }
}

impl Drop for StripWorker {
    fn drop(&mut self) {
        self.shared.stopped.store(true, Ordering::SeqCst);
        kill_child(&self.shared);
        let _ = self.make.send(());
    }
}

fn kill_child(shared: &Shared) {
    let pid = shared.child.load(Ordering::SeqCst);
    if pid != 0 {
        // SAFETY: plain signal to the renderer this worker started.
        unsafe {
            libc::kill(pid as libc::pid_t, libc::SIGKILL);
        }
    }
}

fn load_loop(shared: &Shared, requests: &Receiver<Load>) {
    while let Ok(request) = requests.recv() {
        if shared.stopped.load(Ordering::SeqCst) {
            return;
        }
        match request {
            Load::Middles(names) => {
                for name in names {
                    let frames = std::fs::read(shared.path(&name))
                        .ok()
                        .and_then(|bytes| decode_strip(&bytes));
                    match frames {
                        Some(mut frames) => shared.send(StripEvent::Middle {
                            name,
                            pixels: Arc::new(frames.swap_remove(STRIP_MIDDLE)),
                        }),
                        None => {
                            // A broken file: render it again next time.
                            let _ = std::fs::remove_file(shared.path(&name));
                        }
                    }
                }
            }
            Load::Frames(name) => {
                if let Some(frames) =
                    std::fs::read(shared.path(&name)).ok().and_then(|bytes| decode_strip(&bytes))
                {
                    shared.send(StripEvent::Frames { name, frames: Arc::new(frames) });
                }
            }
        }
    }
}

fn make_loop(shared: &Shared, wakeups: &Receiver<()>, renderer: &Path, stock: &(String, String)) {
    loop {
        if shared.stopped.load(Ordering::SeqCst) {
            return;
        }
        let job = if shared.paused.load(Ordering::SeqCst) {
            None
        } else {
            let mut queue = shared.queue.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if queue.is_empty() { None } else { Some(queue.remove(0)) }
        };
        let Some(job) = job else {
            // Nothing to do (or paused): sleep until told otherwise.
            if wakeups.recv().is_err() {
                return;
            }
            continue;
        };
        if shared.path(&job.name).is_file() {
            continue;
        }
        match render(shared, renderer, stock, &job) {
            Some(bytes) => {
                let target = shared.path(&job.name);
                let temporary = target.with_extension("tmp");
                if std::fs::write(&temporary, bytes).is_ok()
                    && std::fs::rename(&temporary, &target).is_ok()
                {
                    shared.send(StripEvent::Made { name: job.name });
                }
            }
            None => {
                let interrupted =
                    shared.paused.load(Ordering::SeqCst) || shared.stopped.load(Ordering::SeqCst);
                if interrupted {
                    // Stopped on purpose: it stays first in line.
                    shared
                        .queue
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .insert(0, job);
                } else {
                    log::warn!("settings strips: the renderer drew nothing for {}", job.shader);
                    shared.send(StripEvent::Failed { name: job.name });
                }
            }
        }
    }
}

/// Runs the renderer once for `job` and returns the strip file's bytes.
fn render(
    shared: &Shared,
    renderer: &Path,
    stock: &(String, String),
    job: &StripJob,
) -> Option<Vec<u8>> {
    let mut child = Command::new(renderer)
        .arg("*")
        .arg(&stock.1)
        .arg("--transition-from")
        .arg(&stock.0)
        .arg("--shader")
        .arg(&job.shader)
        .arg("--fill-mode")
        .arg("fill")
        .arg("--preview-stream")
        .arg("--preview-size")
        .arg(format!("{STRIP_W}x{STRIP_H}"))
        .arg("--preview-once")
        .arg("--stream-no-header")
        .arg("--duration-ms")
        .arg("1000")
        .env("SKWD_PAPER_PREVIEW_TEST", format!("steps={}", STRIP_FRAMES + 1))
        .env("SKWD_PAPER_TRANSITION_ORIGIN", "default")
        .envs(job.env.iter().map(|(name, value)| (name.as_str(), value.as_str())))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    shared.child.store(child.id(), Ordering::SeqCst);
    // SAFETY: lowers the priority of the renderer just started (nice 19),
    // so it never competes with the picker or the desktop.
    unsafe {
        libc::setpriority(libc::PRIO_PROCESS, child.id() as libc::id_t, 19);
    }
    let mut raw = Vec::with_capacity(FRAME_BYTES * (STRIP_FRAMES + 2));
    let read = child.stdout.take().map(|mut out| out.read_to_end(&mut raw));
    let _ = child.kill();
    let _ = child.wait();
    shared.child.store(0, Ordering::SeqCst);
    read?.ok()?;
    encode_strip(&kept_frames(&raw)?)
}

mod tests;
