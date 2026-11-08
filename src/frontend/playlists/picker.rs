use iced::widget::{column, container, mouse_area, scrollable};
use iced::{Alignment, Element, Length};

use crate::app::Message;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{folio_action, folio_ghost_field, label, row};
use crate::i18n::{tr, tr_args};

use super::{PlMsg, Playlists};

pub(super) fn view<'a>(
    playlists: &'a Playlists,
    viewport: (f32, f32),
    window: crate::frontend::ui::SheetWindow,
    palette: &'a Palette,
) -> Element<'a, Message> {
    let scale = window.scale();
    let (width, height) = window.fit(
        viewport,
        (
            (440.0 * scale).min((viewport.0 - 32.0).max(1.0)),
            (480.0 * scale).min((viewport.1 - 32.0).max(1.0)),
        ),
    );
    let query = playlists.search.trim().to_lowercase();
    let mut choices = column![folio_action(
        tr("playlists-browse-all"),
        playlists.selected.is_none(),
        Some(Message::Pl(PlMsg::Browse(None))),
        Length::Fill,
        scale,
        palette,
    )]
    .spacing(5.0 * scale);
    let mut matches = 0;
    for playlist in &playlists.lists {
        if !query.is_empty()
            && !playlist.name.to_lowercase().contains(&query)
            && !playlist.id.to_string().contains(&query)
        {
            continue;
        }
        matches += 1;
        let name = if playlist.name.is_empty() { tr("playlists-unnamed") } else { &playlist.name };
        choices = choices.push(folio_action(
            tr_args!("playlists-picker-entry", name => name, id => playlist.id.to_string()),
            playlists.selected == Some(playlist.id),
            Some(Message::Pl(PlMsg::Browse(Some(playlist.id)))),
            Length::Fill,
            scale,
            palette,
        ));
    }
    if matches == 0 {
        choices = choices.push(label(
            tr(if playlists.lists.is_empty() {
                "playlists-index-empty-title"
            } else {
                "playlists-picker-no-match"
            }),
            12.0,
            scale,
            palette.surface_text,
        ));
    }
    let body = column![
        row![
            label(tr("playlists-choose"), 16.0, scale, palette.primary).width(Length::Fill),
            folio_action("×", false, Some(Message::ClosePlaylists), Length::Shrink, scale, palette),
        ]
        .align_y(Alignment::Center),
        label(tr("playlists-picker-hint"), 11.0, scale, palette.surface_text),
        folio_ghost_field(
            &playlists.search,
            tr("playlists-picker-search"),
            |value| Message::Pl(PlMsg::Search(value)),
            Message::Noop,
            Length::Fill,
            scale,
            palette,
        ),
        scrollable(choices)
            .height(Length::Fill)
            .direction(crate::frontend::ui::thin_vbar())
            .style(crate::frontend::ui::scroll_style(palette.primary)),
        folio_action(
            tr("playlists-edit"),
            false,
            Some(Message::Pl(PlMsg::EditPlaylists)),
            Length::Fill,
            scale,
            palette
        ),
    ]
    .spacing(12.0 * scale);
    let panel = container(body)
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .padding(18.0 * scale)
        .style(move |_| {
            crate::frontend::ui::box_style(
                crate::frontend::ui::with_alpha(palette.surface, 1.0),
                palette.outline,
            )
        });
    let scrim = window
        .align(container(mouse_area(panel).on_press(Message::Noop)))
        .style(|_| crate::frontend::ui::folio_scrim_style(1.0));
    mouse_area(scrim).on_press(Message::ClosePlaylists).into()
}
