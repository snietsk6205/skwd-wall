use crate::infrastructure::preview::{PREVIEW_H, PREVIEW_W, TransitionPreviewWorker};

#[derive(Default)]
pub(crate) struct TransitionPreviewState {
    worker: TransitionPreviewWorker,
    pub(crate) handle: Option<iced::advanced::image::Handle>,
    /// Custom build (M3): the transition ▶ Preview asked for (a key, "combo"
    /// or a combination name); `None` previews the chosen transition.
    pub(crate) focus: Option<String>,
}

impl TransitionPreviewState {
    pub(crate) fn stop(&mut self) {
        self.worker.stop();
        self.handle = None;
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
        self.worker.ensure(from, to, shader, fill_mode, duration_ms, frame_ms, env);
    }

    pub(crate) fn take_frame(&mut self) -> bool {
        let Some(pixels) = self.worker.take_frame() else {
            return false;
        };
        self.handle = Some(iced::advanced::image::Handle::from_rgba(PREVIEW_W, PREVIEW_H, pixels));
        true
    }
}

#[cfg(test)]
mod tests;
