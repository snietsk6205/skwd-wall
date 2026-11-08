use super::tag_intent_message;
use crate::app::Message;
use crate::frontend::tagcloud::{TagIntent, TagMsg};

#[test]
fn tag_cloud_intents_translate_at_the_app_boundary() {
    assert!(matches!(
        tag_intent_message(TagIntent::Update(TagMsg::Autocomplete)),
        Message::Tag(TagMsg::Autocomplete)
    ));
    assert!(matches!(tag_intent_message(TagIntent::Clear), Message::ClearTags));
    assert!(matches!(tag_intent_message(TagIntent::ToggleCloud), Message::OpenTagCloud));
}

#[test]
fn tag_cloud_docks_where_the_settings_window_sits() {
    use crate::frontend::ui::{SheetPosition, SheetWindow};
    let screen = (1280.0, 720.0);
    let size = (600.0, 126.0);
    let at = |position| SheetWindow::new(1.0, 0.0, 0.0, position);
    assert_eq!(super::cloud_dock(&at(SheetPosition::Center), screen, size, (0.0, 0.0)), None);
    assert_eq!(
        super::cloud_dock(&at(SheetPosition::Top), screen, size, (0.0, 0.0)),
        Some((340.0, 11.0))
    );
    assert_eq!(
        super::cloud_dock(&at(SheetPosition::Bottom), screen, size, (0.0, 0.0)),
        Some((340.0, 583.0))
    );
    assert_eq!(
        super::cloud_dock(&at(SheetPosition::Right), screen, size, (40.0, -20.0)),
        Some((680.0, 277.0))
    );
    // The tag cloud's own offset never pushes it off the screen.
    assert_eq!(
        super::cloud_dock(&at(SheetPosition::Left), screen, size, (-300.0, 900.0)),
        Some((0.0, 594.0))
    );
}
