use iced::widget::{column, container, mouse_area, scrollable};
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{folio_action, label, row};
use crate::i18n::tr;

use super::PlMsg;

pub(super) fn view(
    viewport: (f32, f32),
    window: crate::frontend::ui::SheetWindow,
    palette: &Palette,
) -> Element<'_, Message> {
    let scale = window.scale();
    let (sheet_width, sheet_height) = window.dims(viewport);
    let mut entries =
        column![label(tr("playlists-help-combine"), 12.0, scale, palette.surface_text)]
            .spacing(14.0 * scale);
    for key in [
        "playlists-help-source",
        "playlists-help-folder",
        "playlists-help-type",
        "playlists-help-tag",
        "playlists-help-colour",
        "playlists-help-ratio",
        "playlists-help-width",
        "playlists-help-height",
        "playlists-help-resolution",
        "playlists-help-comparison",
        "playlists-help-example",
    ] {
        entries =
            entries.push(label(tr(key), 12.0, scale, palette.surface_text).width(Length::Fill));
    }
    let close = Message::Pl(PlMsg::FilterHelp(false));
    let body = column![
        row![
            label(tr("playlists-filter-help"), 16.0, scale, palette.primary).width(Length::Fill),
            folio_action("×", false, Some(close.clone()), Length::Shrink, scale, palette),
        ]
        .align_y(Alignment::Center),
        scrollable(container(entries).padding([0.0, 8.0]))
            .height(Length::Fill)
            .direction(crate::frontend::ui::thin_vbar())
            .style(crate::frontend::ui::scroll_style(palette.primary)),
    ]
    .spacing(16.0 * scale);
    let panel = container(body)
        .width(Length::Fixed((560.0 * scale).min(sheet_width).min((viewport.0 - 32.0).max(1.0))))
        .height(Length::Fixed((560.0 * scale).min(sheet_height).min((viewport.1 - 32.0).max(1.0))))
        .padding(20.0 * scale)
        .style(move |_| {
            crate::frontend::ui::box_style(
                crate::frontend::ui::with_alpha(palette.surface, 1.0),
                palette.outline,
            )
        });
    let scrim = window
        .align(container(mouse_area(panel).on_press(Message::Noop)))
        .style(|_| crate::frontend::ui::folio_scrim_style(1.0));
    mouse_area(scrim).on_press(close).into()
}
