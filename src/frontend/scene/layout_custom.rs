//! Custom build: parameters of the Coverflow and Showcase layouts.

use super::layout::{feq, lerp};

/// How the Coverflow layout arranges its cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CoverflowStyle {
    /// The selected card faces you; the others turn away on both sides.
    #[default]
    Coverflow,
    /// The cards stand on a turning ring, like a carousel.
    Carousel,
    /// The cards recede into the distance behind the selected one.
    Stack,
}

impl CoverflowStyle {
    pub fn from_key(value: &str) -> Self {
        match value {
            "carousel" | "ring" => Self::Carousel,
            "stack" | "timeline" => Self::Stack,
            _ => Self::Coverflow,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CoverflowParams {
    pub style: CoverflowStyle,
    /// Card height in percent of the screen height.
    pub size: f32,
    /// Angle of the side cards in degrees.
    pub angle: f32,
    /// Distance between side cards in percent of the card width.
    pub spacing: f32,
    /// Cards shown on each side of the selected one.
    pub count: f32,
    pub reflection: bool,
    pub backdrop: bool,
    pub corners: f32,
    /// Time for one step, in milliseconds.
    pub speed_ms: f32,
}

impl Default for CoverflowParams {
    fn default() -> Self {
        Self {
            style: CoverflowStyle::Coverflow,
            size: 46.0,
            angle: 62.0,
            spacing: 26.0,
            count: 5.0,
            reflection: true,
            backdrop: true,
            corners: 10.0,
            speed_ms: 420.0,
        }
    }
}

impl CoverflowParams {
    pub(crate) fn morph_toward(&mut self, target: &Self, amt: f32) {
        self.style = target.style;
        self.size = lerp(self.size, target.size, amt);
        self.angle = lerp(self.angle, target.angle, amt);
        self.spacing = lerp(self.spacing, target.spacing, amt);
        self.count = lerp(self.count, target.count, amt);
        self.reflection = target.reflection;
        self.backdrop = target.backdrop;
        self.corners = lerp(self.corners, target.corners, amt);
        self.speed_ms = lerp(self.speed_ms, target.speed_ms, amt);
    }

    pub(crate) fn settled_to(&self, target: &Self) -> bool {
        self.style == target.style
            && feq(self.size, target.size)
            && feq(self.angle, target.angle)
            && feq(self.spacing, target.spacing)
            && feq(self.count, target.count)
            && self.reflection == target.reflection
            && self.backdrop == target.backdrop
            && feq(self.corners, target.corners)
            && feq(self.speed_ms, target.speed_ms)
    }
}

/// Where the Showcase filmstrip sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StripPosition {
    #[default]
    Bottom,
    Top,
}

impl StripPosition {
    pub fn from_key(value: &str) -> Self {
        if value == "top" { Self::Top } else { Self::Bottom }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ShowcaseParams {
    /// Width of the big preview in percent of the screen width.
    pub hero: f32,
    /// Height of the filmstrip cards in pixels.
    pub strip: f32,
    pub strip_position: StripPosition,
    pub backdrop: bool,
    pub corners: f32,
    /// Time for one step, in milliseconds.
    pub speed_ms: f32,
}

impl Default for ShowcaseParams {
    fn default() -> Self {
        Self {
            hero: 62.0,
            strip: 92.0,
            strip_position: StripPosition::Bottom,
            backdrop: true,
            corners: 14.0,
            speed_ms: 380.0,
        }
    }
}

impl ShowcaseParams {
    pub(crate) fn morph_toward(&mut self, target: &Self, amt: f32) {
        self.hero = lerp(self.hero, target.hero, amt);
        self.strip = lerp(self.strip, target.strip, amt);
        self.strip_position = target.strip_position;
        self.backdrop = target.backdrop;
        self.corners = lerp(self.corners, target.corners, amt);
        self.speed_ms = lerp(self.speed_ms, target.speed_ms, amt);
    }

    pub(crate) fn settled_to(&self, target: &Self) -> bool {
        feq(self.hero, target.hero)
            && feq(self.strip, target.strip)
            && self.strip_position == target.strip_position
            && self.backdrop == target.backdrop
            && feq(self.corners, target.corners)
            && feq(self.speed_ms, target.speed_ms)
    }
}
