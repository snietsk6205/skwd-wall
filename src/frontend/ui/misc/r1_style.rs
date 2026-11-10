//! R1 (custom build): the few style shapes the redesigned Settings window
//! uses (rounded surfaces, pills, gradient scrims), kept in the shared UI
//! module like every other style.

use iced::widget::{button, container};
use iced::{Background, Border, Color, Shadow, Vector};

/// A rounded surface: fill, corner radius, border (width, colour) and an
/// optional drop shadow (colour, y offset, blur).
pub fn surface_style(
    fill: Color,
    radius: f32,
    border: (f32, Color),
    shadow: Option<(Color, f32, f32)>,
) -> container::Style {
    container::Style {
        background: Some(Background::Color(fill)),
        border: Border { radius: radius.into(), width: border.0, color: border.1 },
        shadow: shadow.map_or_else(Shadow::default, |(color, y, blur)| Shadow {
            color,
            offset: Vector::new(0.0, y),
            blur_radius: blur,
        }),
        ..Default::default()
    }
}

/// A rounded button: fill, corner radius, border (width, colour), text colour
/// and an optional drop shadow.
pub fn rounded_button_style(
    fill: Color,
    radius: f32,
    border: (f32, Color),
    text: Color,
    shadow: Option<(Color, f32, f32)>,
) -> button::Style {
    button::Style {
        background: Some(Background::Color(fill)),
        border: Border { radius: radius.into(), width: border.0, color: border.1 },
        text_color: text,
        shadow: shadow.map_or_else(Shadow::default, |(color, y, blur)| Shadow {
            color,
            offset: Vector::new(0.0, y),
            blur_radius: blur,
        }),
        ..Default::default()
    }
}

fn alpha(color: Color, a: f32) -> Color {
    Color { a, ..color }
}

/// Text-like button of the top bar, menus and palette: no fill until
/// hovered; a ring when the keyboard is on it; primary text when active.
pub fn ghost_button_style(
    palette: &crate::frontend::theme::Palette,
    fade: f32,
    active: bool,
    focused: bool,
    status: button::Status,
) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    let fill = if focused {
        0.16
    } else if hovered {
        0.09
    } else {
        0.0
    };
    rounded_button_style(
        alpha(palette.surface_text, fill * fade),
        8.0,
        (if focused { 1.0 } else { 0.0 }, alpha(palette.primary, 0.8 * fade)),
        alpha(
            if active { palette.primary } else { palette.surface_text },
            if active || hovered || focused { fade } else { 0.78 * fade },
        ),
        None,
    )
}

/// Small square-ish button that only shows a fill on hover (reset, close).
pub fn soft_button_style(
    palette: &crate::frontend::theme::Palette,
    fade: f32,
    status: button::Status,
) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    rounded_button_style(
        alpha(palette.surface_text, if hovered { 0.12 * fade } else { 0.0 }),
        6.0,
        (0.0, Color::TRANSPARENT),
        alpha(palette.surface_text, fade),
        None,
    )
}

/// A Settings row: a soft tint and ring while the keyboard is on it.
pub fn focus_row_style(
    palette: &crate::frontend::theme::Palette,
    fade: f32,
    focused: bool,
) -> container::Style {
    surface_style(
        alpha(palette.primary, if focused { 0.08 * fade } else { 0.0 }),
        8.0,
        (if focused { 1.0 } else { 0.0 }, alpha(palette.primary, 0.55 * fade)),
        None,
    )
}
