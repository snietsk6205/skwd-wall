use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::super::ipc::IpcMsg;

#[derive(Debug, Clone)]
pub enum Wake {
    Frame(Instant),
    Ipc(IpcMsg),
    Decoded,
    SelfTest(u32),
    HudTick,
    Toggle,
    Hide,
    Command(String),
    Query(#[allow(dead_code)] String, Reply),
    EffectSrc {
        card: usize,
        source: String,
        rgba: Arc<Vec<u8>>,
        w: u32,
        h: u32,
    },
    Semantic(crate::infrastructure::semantic::SemanticResult),
    /// R1b: a frame strip for the Settings cards was read or rendered.
    Strip(crate::infrastructure::preview::StripEvent),
    /// R1: the Settings Home background, decoded (sharp, blurred).
    HomeBackdrop {
        source: String,
        #[allow(clippy::type_complexity)]
        decoded: Option<
            Box<(
                crate::infrastructure::preview::BackdropPixels,
                crate::infrastructure::preview::BackdropPixels,
            )>,
        >,
    },
}

type ReplyFn = Box<dyn FnOnce(&str) + Send>;

#[derive(Clone)]
pub struct Reply(Arc<Mutex<Option<ReplyFn>>>);

impl Reply {
    pub fn new(func: ReplyFn) -> Self {
        Self(Arc::new(Mutex::new(Some(func))))
    }

    pub fn send(&self, response: &str) {
        if let Some(func) = self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take()
        {
            func(response);
        }
    }
}

impl std::fmt::Debug for Reply {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.write_str("Reply")
    }
}
