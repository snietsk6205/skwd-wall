use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

pub(crate) const PREVIEW_W: u32 = 640;
pub(crate) const PREVIEW_H: u32 = 360;

#[derive(Default)]
struct PreviewFrame {
    generation: u64,
    pixels: Option<Vec<u8>>,
}

type Shared = Arc<Mutex<PreviewFrame>>;

pub(crate) struct TransitionPreviewWorker {
    child: Option<Child>,
    latest: Shared,
    key: String,
    seen: usize,
}

impl Default for TransitionPreviewWorker {
    fn default() -> Self {
        Self {
            child: None,
            latest: Arc::new(Mutex::new(PreviewFrame::default())),
            key: String::new(),
            seen: 0,
        }
    }
}

const STOCK_A: (&str, &[u8]) =
    ("Tori-Jade.webp", include_bytes!("../../../data/preview/Tori-Jade.webp"));
const STOCK_B: (&str, &[u8]) =
    ("Flower-3.webp", include_bytes!("../../../data/preview/Flower-3.webp"));

pub(crate) fn stock_pair(cache_dir: &str) -> Option<(String, String)> {
    let dir = std::path::Path::new(cache_dir).join("transition-preview");
    let mut out = Vec::with_capacity(2);
    for (name, bytes) in [STOCK_A, STOCK_B] {
        let path = dir.join(name);
        if path.metadata().map(|meta| meta.len()).ok() != Some(bytes.len() as u64) {
            std::fs::create_dir_all(&dir).ok()?;
            std::fs::write(&path, bytes).ok()?;
        }
        out.push(path.to_string_lossy().into_owned());
    }
    Some((out[0].clone(), out[1].clone()))
}

impl TransitionPreviewWorker {
    pub(crate) fn stop(&mut self) {
        {
            let mut latest = self.latest.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            latest.generation = latest.generation.wrapping_add(1);
            latest.pixels = None;
        }
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.key.clear();
        self.seen = 0;
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn ensure(
        &mut self,
        from: &str,
        to: &str,
        shader: &str,
        fill_mode: &str,
        duration_ms: u64,
        frame_ms: u64,
        env: &[(&str, String)],
    ) {
        let env_key: String = env
            .iter()
            .map(|(name, value)| [*name, "=", value.as_str(), "\u{0}"].concat())
            .collect();
        let key = format!(
            "{shader}\u{0}{fill_mode}\u{0}{duration_ms}\u{0}{frame_ms}\u{0}{from}\u{0}{to}\u{0}{env_key}"
        );
        if self.key == key && self.child.is_some() {
            return;
        }
        self.stop();
        if from.is_empty() || to.is_empty() {
            return;
        }
        let bin = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.join("skwd-wall-vk")))
            .filter(|path| path.exists())
            .unwrap_or_else(|| std::path::PathBuf::from("skwd-wall-vk"));
        let spawned = Command::new(bin)
            .arg("*")
            .arg(to)
            .arg("--transition-from")
            .arg(from)
            .arg("--shader")
            .arg(shader)
            .arg("--fill-mode")
            .arg(fill_mode)
            .arg("--preview-stream")
            .arg("--preview-size")
            .arg(format!("{PREVIEW_W}x{PREVIEW_H}"))
            .arg("--duration-ms")
            .arg(duration_ms.max(300).to_string())
            .arg("--preview-frame-ms")
            .arg(frame_ms.to_string())
            .envs(env.iter().map(|(name, value)| (*name, value.as_str())))
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();
        let Ok(mut child) = spawned else {
            log::warn!("transition preview: could not start the renderer");
            return;
        };
        log::info!(
            "transition preview: spawned pid {} shader={shader} {duration_ms}ms from={from} to={to}",
            child.id()
        );
        let Some(mut out) = child.stdout.take() else {
            let _ = child.kill();
            let _ = child.wait();
            return;
        };
        let latest = Arc::clone(&self.latest);
        let generation =
            latest.lock().unwrap_or_else(std::sync::PoisonError::into_inner).generation;
        let reader = std::thread::Builder::new().name("preview-stream".into()).spawn(move || {
            let mut header = [0u8; 12];
            if out.read_exact(&mut header).is_err() || &header[0..4] != b"SKWP" {
                return;
            }
            let w = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
            let h = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
            if (w, h) != (PREVIEW_W, PREVIEW_H) {
                log::warn!("transition preview: rejected unexpected stream dimensions {w}x{h}");
                return;
            }
            log::info!("transition preview: stream header {w}x{h}");
            let mut buf = vec![0u8; PREVIEW_W as usize * PREVIEW_H as usize * 4];
            let mut got = 0u64;
            while out.read_exact(&mut buf).is_ok() {
                got += 1;
                if got.is_multiple_of(120) {
                    log::debug!("transition preview: {got} frames read");
                }
                if !publish_frame(&latest, generation, &buf) {
                    break;
                }
            }
            log::info!("transition preview: stream ended after {got} frames");
        });
        if let Err(error) = reader {
            log::warn!("transition preview: could not start stream reader: {error}");
            let _ = child.kill();
            let _ = child.wait();
            return;
        }
        self.child = Some(child);
        self.key = key;
    }

    pub(crate) fn take_frame(&mut self) -> Option<Vec<u8>> {
        let frame =
            self.latest.lock().unwrap_or_else(std::sync::PoisonError::into_inner).pixels.take();
        let pixels = frame?;
        self.seen = self.seen.wrapping_add(1);
        if self.seen % 120 == 1 {
            log::info!("transition preview: {} frames delivered", self.seen);
        }
        Some(pixels)
    }
}

fn publish_frame(latest: &Shared, generation: u64, pixels: &[u8]) -> bool {
    let mut slot = latest.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if slot.generation != generation {
        return false;
    }
    match slot.pixels.as_mut() {
        Some(previous) => previous.copy_from_slice(pixels),
        None => slot.pixels = Some(pixels.to_vec()),
    }
    true
}

impl Drop for TransitionPreviewWorker {
    fn drop(&mut self) {
        self.stop();
    }
}

mod tests;
