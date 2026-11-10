//! R1b (custom build): the look of the redesigned Settings window. A calm,
//! always-dark sheet in neutral inks with thin lines, and one accent colour
//! taken from the theme. Every style shape of that look lives here.

use iced::widget::canvas::{self, Frame, Path, Stroke};
use iced::widget::{button, container, text_input};
use iced::{Background, Border, Color, Element, Length, Point, Rectangle, Shadow, Vector, mouse};

use crate::frontend::theme::Palette;

/// The sheet behind every page.
pub const INK: Color = Color::from_rgb(0.043, 0.047, 0.055);
/// Menus, filled cards and the big preview's frame.
const INK_RAISED: Color = Color::from_rgb(0.071, 0.075, 0.086);
/// Tracks of sliders and switches, hovered fills.
const INK_SOFT: Color = Color::from_rgb(0.118, 0.125, 0.141);
/// Text. Slightly warm so it does not glare on the ink.
const PAPER: Color = Color::from_rgb(0.925, 0.925, 0.902);
/// Lines (always used at a low opacity).
const LINE: Color = Color::from_rgb(0.62, 0.635, 0.66);

fn alpha(color: Color, a: f32) -> Color {
    Color { a, ..color }
}

fn mix(from: Color, to: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color {
        r: from.r + (to.r - from.r) * t,
        g: from.g + (to.g - from.g) * t,
        b: from.b + (to.b - from.b) * t,
        a: from.a + (to.a - from.a) * t,
    }
}

fn to_hsl(color: Color) -> (f32, f32, f32) {
    let (r, g, b) = (color.r, color.g, color.b);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let light = f32::midpoint(max, min);
    let span = max - min;
    if span < 1e-5 {
        return (0.0, 0.0, light);
    }
    let saturation = span / (1.0 - (2.0 * light - 1.0).abs()).max(1e-5);
    let hue = if (max - r).abs() < f32::EPSILON {
        ((g - b) / span).rem_euclid(6.0)
    } else if (max - g).abs() < f32::EPSILON {
        (b - r) / span + 2.0
    } else {
        (r - g) / span + 4.0
    };
    (hue * 60.0, saturation.clamp(0.0, 1.0), light)
}

fn from_hsl(hue: f32, saturation: f32, light: f32) -> Color {
    let chroma = (1.0 - (2.0 * light - 1.0).abs()) * saturation;
    let sector = hue.rem_euclid(360.0) / 60.0;
    let second = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());
    let (r, g, b) = match sector as u32 {
        0 => (chroma, second, 0.0),
        1 => (second, chroma, 0.0),
        2 => (0.0, chroma, second),
        3 => (0.0, second, chroma),
        4 => (second, 0.0, chroma),
        _ => (chroma, 0.0, second),
    };
    let lift = light - chroma / 2.0;
    Color::from_rgb(r + lift, g + lift, b + lift)
}

/// The theme's main colour, made light and clear enough to read on the ink
/// (a dark or greyish theme colour would vanish on it).
pub fn accent_on_ink(theme: Color) -> Color {
    let (hue, saturation, light) = to_hsl(theme);
    if saturation < 0.08 {
        // A grey theme keeps a grey (paper) accent.
        return from_hsl(hue, saturation, light.max(0.82));
    }
    from_hsl(hue, saturation.clamp(0.45, 0.92), light.clamp(0.68, 0.8))
}

/// The Settings palette: neutral inks, paper text, and the theme's main
/// colour as the only accent. The old controls read it like any palette.
pub fn ink_palette(theme: &Palette) -> Palette {
    let accent = accent_on_ink(theme.primary);
    Palette {
        primary: accent,
        primary_text: INK,
        surface: INK,
        surface_text: PAPER,
        surface_variant: INK_SOFT,
        surface_container: INK_RAISED,
        background: INK,
        outline: LINE,
        tertiary: accent,
    }
}

/// A plain fill (the sheet, scrims over pictures).
pub fn ink_fill(color: Color, opacity: f32) -> container::Style {
    container::Style {
        background: Some(Background::Color(alpha(color, opacity))),
        ..Default::default()
    }
}

/// A one-pixel line (rules, the top bar's underline).
pub fn hairline_style(palette: &Palette, opacity: f32) -> container::Style {
    ink_fill(palette.outline, opacity)
}

/// The outline of a card. `lit` (0..1) moves the line from quiet to the
/// accent (hover); `focused` is the keyboard ring; `filled` gives the card
/// a raised fill instead of the bare sheet.
pub fn tile_style(
    palette: &Palette,
    fade: f32,
    lit: f32,
    focused: bool,
    filled: bool,
) -> container::Style {
    let quiet = alpha(palette.outline, 0.2 * fade);
    let bright = alpha(palette.primary, 0.95 * fade);
    let line = if focused { alpha(palette.primary, fade) } else { mix(quiet, bright, lit) };
    let base = if filled { 0.92 } else { 0.0 };
    container::Style {
        background: Some(Background::Color(mix(
            alpha(palette.surface_container, base * fade),
            alpha(palette.surface_variant, (base.max(0.55)) * fade),
            if focused { 1.0 } else { lit } * 0.6,
        ))),
        border: Border {
            radius: (3.0).into(),
            width: if focused { 1.5 } else { 1.0 },
            color: line,
        },
        shadow: if focused {
            Shadow {
                color: alpha(palette.primary, 0.22 * fade),
                offset: Vector::new(0.0, 0.0),
                blur_radius: 14.0,
            }
        } else {
            Shadow::default()
        },
        ..Default::default()
    }
}

/// A card that is one big button (menu pages, Home shortcuts).
pub fn tile_button_style(
    palette: &Palette,
    fade: f32,
    focused: bool,
    marked: bool,
    filled: bool,
    status: button::Status,
) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    let lit = if hovered { 1.0 } else { 0.0 };
    let card = tile_style(palette, fade, lit, focused, filled);
    let mut border = card.border;
    if marked && !focused && !hovered {
        border.color = alpha(palette.outline, 0.42 * fade);
    }
    button::Style {
        background: card.background,
        border,
        shadow: card.shadow,
        text_color: alpha(palette.surface_text, fade),
        ..Default::default()
    }
}

/// A small tag at the bottom of a card (family, favourite, playing now).
/// `strong` uses the accent.
pub fn tag_style(palette: &Palette, fade: f32, strong: bool) -> container::Style {
    container::Style {
        background: Some(Background::Color(alpha(
            palette.primary,
            if strong { 0.04 * fade } else { 0.0 },
        ))),
        border: Border {
            radius: (2.0).into(),
            width: 1.0,
            color: if strong {
                alpha(palette.primary, 0.6 * fade)
            } else {
                alpha(palette.outline, 0.26 * fade)
            },
        },
        ..Default::default()
    }
}

/// Text that is a button (top bar, breadcrumb, filters): never a fill, the
/// text brightens on hover and the keyboard shows a thin ring.
pub fn text_button_style(
    palette: &Palette,
    fade: f32,
    active: bool,
    focused: bool,
    status: button::Status,
) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    let strength = if active {
        1.0
    } else if hovered || focused {
        0.86
    } else {
        0.46
    };
    button::Style {
        background: None,
        border: Border {
            radius: (3.0).into(),
            width: if focused { 1.0 } else { 0.0 },
            color: alpha(palette.primary, 0.9 * fade),
        },
        text_color: alpha(palette.surface_text, strength * fade),
        ..Default::default()
    }
}

/// An action on a lit card. `primary` is the accent-filled one; the others
/// are outlined and fill on hover.
pub fn tile_action_style(
    palette: &Palette,
    fade: f32,
    primary: bool,
    focused: bool,
    status: button::Status,
) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    let (fill, text, line) = if primary {
        (
            alpha(palette.primary, if hovered || focused { fade } else { 0.9 * fade }),
            alpha(palette.primary_text, fade),
            alpha(palette.primary, fade),
        )
    } else if hovered || focused {
        (
            alpha(palette.surface_text, 0.94 * fade),
            alpha(palette.primary_text, fade),
            alpha(palette.surface_text, fade),
        )
    } else {
        (
            Color::TRANSPARENT,
            alpha(palette.surface_text, 0.9 * fade),
            alpha(palette.surface_text, 0.42 * fade),
        )
    };
    button::Style {
        background: Some(Background::Color(fill)),
        border: Border { radius: (2.0).into(), width: 1.0, color: line },
        text_color: text,
        shadow: if focused {
            Shadow {
                color: alpha(palette.primary, 0.5 * fade),
                offset: Vector::new(0.0, 0.0),
                blur_radius: 8.0,
            }
        } else {
            Shadow::default()
        },
        ..Default::default()
    }
}

/// A filter word above a card grid: outlined when chosen.
pub fn filter_button_style(
    palette: &Palette,
    fade: f32,
    chosen: bool,
    status: button::Status,
) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    button::Style {
        background: Some(Background::Color(alpha(
            palette.primary,
            if chosen { 0.04 * fade } else { 0.0 },
        ))),
        border: Border {
            radius: (2.0).into(),
            width: 1.0,
            color: if chosen {
                alpha(palette.primary, 0.7 * fade)
            } else {
                alpha(palette.outline, if hovered { 0.5 } else { 0.0 } * fade)
            },
        },
        text_color: alpha(
            palette.surface_text,
            if chosen {
                fade
            } else if hovered {
                0.9 * fade
            } else {
                0.52 * fade
            },
        ),
        ..Default::default()
    }
}

/// A floating panel (group menus, the palette, the big preview's frame).
pub fn floating_panel_style(palette: &Palette, fade: f32) -> container::Style {
    container::Style {
        background: Some(Background::Color(alpha(palette.surface, fade))),
        border: Border {
            radius: (4.0).into(),
            width: 1.0,
            color: alpha(palette.outline, 0.24 * fade),
        },
        shadow: Shadow {
            color: alpha(Color::BLACK, 0.55 * fade),
            offset: Vector::new(0.0, 18.0),
            blur_radius: 48.0,
        },
        ..Default::default()
    }
}

/// The Settings window itself.
pub fn sheet_style(palette: &Palette, fade: f32) -> container::Style {
    container::Style {
        background: Some(Background::Color(palette.surface)),
        border: Border {
            radius: (6.0).into(),
            width: 1.0,
            color: alpha(palette.outline, 0.2 * fade),
        },
        shadow: Shadow {
            color: alpha(Color::BLACK, 0.5 * fade),
            offset: Vector::new(0.0, 16.0),
            blur_radius: 44.0,
        },
        ..Default::default()
    }
}

/// A quiet text field: a thin outline that takes the accent while typing.
pub fn quiet_input_style(
    palette: &Palette,
    fade: f32,
    status: text_input::Status,
) -> text_input::Style {
    let focused = matches!(status, text_input::Status::Focused { .. });
    let hovered = matches!(status, text_input::Status::Hovered);
    text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border {
            radius: (2.0).into(),
            width: 1.0,
            color: if focused {
                alpha(palette.primary, 0.85 * fade)
            } else {
                alpha(palette.outline, if hovered { 0.42 } else { 0.24 } * fade)
            },
        },
        icon: alpha(palette.surface_text, 0.5 * fade),
        placeholder: alpha(palette.surface_text, 0.34 * fade),
        value: alpha(palette.surface_text, fade),
        selection: alpha(palette.primary, 0.35 * fade),
    }
}

/// A dark veil over a picture, stronger at the bottom (Home) so text
/// stays readable.
pub fn veil_style(top: f32, middle: f32, bottom: f32) -> container::Style {
    container::Style {
        background: Some(Background::Gradient(iced::Gradient::Linear(
            iced::gradient::Linear::new(std::f32::consts::PI)
                .add_stop(0.0, alpha(INK, top))
                .add_stop(0.5, alpha(INK, middle))
                .add_stop(1.0, alpha(INK, bottom)),
        ))),
        ..Default::default()
    }
}

struct GridPattern {
    color: Color,
    step: f32,
}

impl<Message> canvas::Program<Message> for GridPattern {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let stroke = Stroke::default().with_width(1.0).with_color(self.color);
        let step = self.step.max(8.0);
        // Centred, so the pattern looks the same at every window size.
        let mut x = (bounds.width / 2.0).rem_euclid(step);
        while x < bounds.width {
            let at = x.floor() + 0.5;
            frame.stroke(&Path::line(Point::new(at, 0.0), Point::new(at, bounds.height)), stroke);
            x += step;
        }
        let mut y = step;
        while y < bounds.height {
            let at = y.floor() + 0.5;
            frame.stroke(&Path::line(Point::new(0.0, at), Point::new(bounds.width, at)), stroke);
            y += step;
        }
        vec![frame.into_geometry()]
    }
}

/// A faint square grid that fills its space (behind pages).
pub fn grid_pattern<'a, Message: 'a>(
    palette: &Palette,
    opacity: f32,
    step: f32,
) -> Element<'a, Message> {
    canvas::Canvas::new(GridPattern { color: alpha(palette.outline, opacity), step })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// One choice among several (or an action) in a plain row: just the word.
/// The chosen one gets a thin accent outline; the keyboard a ring; `strong`
/// (a confirmation) fills with the accent.
pub fn choice_button_style(
    palette: &Palette,
    fade: f32,
    chosen: bool,
    strong: bool,
    focused: bool,
    status: button::Status,
) -> button::Style {
    let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
    let disabled = matches!(status, button::Status::Disabled);
    if strong && chosen {
        return button::Style {
            background: Some(Background::Color(alpha(palette.primary, fade))),
            border: Border {
                radius: (2.0).into(),
                width: 1.0,
                color: alpha(palette.primary, fade),
            },
            text_color: alpha(palette.primary_text, fade),
            ..Default::default()
        };
    }
    let line = if focused {
        alpha(palette.primary, fade)
    } else if chosen {
        alpha(palette.primary, 0.7 * fade)
    } else if hovered && !disabled {
        alpha(palette.outline, 0.45 * fade)
    } else {
        Color::TRANSPARENT
    };
    button::Style {
        background: Some(Background::Color(alpha(
            palette.primary,
            if chosen { 0.04 * fade } else { 0.0 },
        ))),
        border: Border {
            radius: (2.0).into(),
            width: if focused { 1.5 } else { 1.0 },
            color: line,
        },
        text_color: alpha(
            palette.surface_text,
            if disabled {
                0.28 * fade
            } else if chosen || hovered || focused {
                fade
            } else {
                0.58 * fade
            },
        ),
        ..Default::default()
    }
}

/// The track of a switch: an outline when off, the accent when on.
pub fn switch_track_style(palette: &Palette, fade: f32, on: bool, lit: bool) -> container::Style {
    container::Style {
        background: Some(Background::Color(alpha(palette.primary, if on { fade } else { 0.0 }))),
        border: Border {
            radius: (999.0).into(),
            width: 1.0,
            color: if on {
                alpha(palette.primary, fade)
            } else {
                alpha(palette.outline, if lit { 0.8 } else { 0.45 } * fade)
            },
        },
        ..Default::default()
    }
}

/// The knob of a switch.
pub fn switch_thumb_style(palette: &Palette, fade: f32, on: bool) -> container::Style {
    container::Style {
        background: Some(Background::Color(if on {
            alpha(palette.primary_text, fade)
        } else {
            alpha(palette.surface_text, 0.62 * fade)
        })),
        border: Border { radius: (999.0).into(), width: 0.0, color: Color::TRANSPARENT },
        ..Default::default()
    }
}

/// A plain Settings row: nothing at rest, a thin accent ring and the
/// faintest tint while the keyboard is on it.
pub fn plain_row_style(palette: &Palette, fade: f32, focused: bool) -> container::Style {
    container::Style {
        background: Some(Background::Color(alpha(
            palette.primary,
            if focused { 0.012 * fade } else { 0.0 },
        ))),
        border: Border {
            radius: (3.0).into(),
            width: if focused { 1.0 } else { 0.0 },
            color: alpha(palette.primary, 0.75 * fade),
        },
        ..Default::default()
    }
}
