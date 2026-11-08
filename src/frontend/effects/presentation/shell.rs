use iced::widget::{container, stack, text};
use iced::{Element, Length};

use crate::app::Message;
use crate::frontend::theme::Palette;
use crate::frontend::ui::with_alpha;

use super::super::PreviewRenderer;
use super::super::state::{Effects, EffectsMode};

impl Effects {
    pub fn view<'a>(
        &'a self,
        viewport: (f32, f32),
        window: crate::frontend::ui::SheetWindow,
        theme: &'a Palette,
        preview_renderer: &impl PreviewRenderer<Message, Output = Element<'static, Message>>,
    ) -> Element<'a, Message> {
        let palette = &self.panel.chrome;
        let body = match self.mode() {
            EffectsMode::Studio => self.studio_view(viewport, window, palette, preview_renderer),
            EffectsMode::Displays => self.displays_view(viewport, window, theme),
        };
        let commit = self.apply_ease().clamp(0.0, 1.0);
        if commit <= 0.003 {
            return body;
        }
        let veil = with_alpha(palette.surface, 0.9 * commit);
        stack![
            body,
            container(text(""))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| crate::frontend::ui::bg_style(veil)),
        ]
        .into()
    }
}
