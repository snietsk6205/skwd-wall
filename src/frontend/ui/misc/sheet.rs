use iced::alignment::{Horizontal, Vertical};
use iced::widget::{Container, container};
use iced::{Element, Length};

const SCREEN_MARGIN: f32 = 22.0;
const MIN_WIDTH: f32 = 220.0;
const MIN_HEIGHT: f32 = 160.0;
const EDGE_GAP: f32 = 11.0;

/// Where a sheet sits on the screen (Settings → Picker → Settings window →
/// Position). Left and right are screen sides, not reading sides.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SheetPosition {
    #[default]
    Center,
    Left,
    Right,
    Top,
    Bottom,
}

impl SheetPosition {
    pub fn parse(value: &str) -> Self {
        match value {
            "left" => Self::Left,
            "right" => Self::Right,
            "top" => Self::Top,
            "bottom" => Self::Bottom,
            _ => Self::Center,
        }
    }
}

/// Size, place and scale shared by the Settings window and every panel that
/// opens over the picker. A share of 0 keeps the panel's own automatic size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SheetWindow {
    scale: f32,
    width_share: f32,
    height_share: f32,
    position: SheetPosition,
}

impl SheetWindow {
    #[cfg(test)]
    pub fn automatic(scale: f32) -> Self {
        Self { scale, width_share: 0.0, height_share: 0.0, position: SheetPosition::Center }
    }

    /// `width_share` and `height_share` are fractions of the screen (0.2 to 1);
    /// anything else means automatic.
    pub fn new(scale: f32, width_share: f32, height_share: f32, position: SheetPosition) -> Self {
        let share = |value: f32| {
            if value.is_finite() && value >= 0.2 { value.min(1.0) } else { 0.0 }
        };
        Self {
            scale: if scale.is_finite() && scale > 0.0 { scale } else { 1.0 },
            width_share: share(width_share),
            height_share: share(height_share),
            position,
        }
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn position(&self) -> SheetPosition {
        self.position
    }

    pub fn has_height(&self) -> bool {
        self.height_share > 0.0
    }

    /// The same window with another scale (a panel that must stay legible).
    pub fn with_scale(self, scale: f32) -> Self {
        Self { scale, ..self }
    }

    /// Size of a full folio sheet (Settings, playlists, schedule, ...).
    pub fn dims(&self, viewport: (f32, f32)) -> (f32, f32) {
        self.fit(viewport, super::folio_sheet_dims(viewport, self.scale))
    }

    /// Size of a panel whose automatic size is `auto`: the window's width and
    /// height replace it when they are set.
    pub fn fit(&self, viewport: (f32, f32), auto: (f32, f32)) -> (f32, f32) {
        (self.width_or(viewport.0, auto.0), self.height_or(viewport.1, auto.1))
    }

    pub fn width_or(&self, screen: f32, auto: f32) -> f32 {
        fit_share(self.width_share, screen, auto, MIN_WIDTH)
    }

    pub fn height_or(&self, screen: f32, auto: f32) -> f32 {
        fit_share(self.height_share, screen, auto, MIN_HEIGHT)
    }

    pub fn align_x(&self) -> Horizontal {
        match self.position {
            SheetPosition::Left => Horizontal::Left,
            SheetPosition::Right => Horizontal::Right,
            _ => Horizontal::Center,
        }
    }

    pub fn align_y(&self) -> Vertical {
        match self.position {
            SheetPosition::Top => Vertical::Top,
            SheetPosition::Bottom => Vertical::Bottom,
            _ => Vertical::Center,
        }
    }

    /// Gap between a sheet and the screen edge it is docked to.
    pub fn edge_gap(&self) -> f32 {
        if self.position == SheetPosition::Center { 0.0 } else { EDGE_GAP * self.scale }
    }

    /// Top-left corner of a `size` panel placed on a `viewport` screen.
    pub fn origin(&self, viewport: (f32, f32), size: (f32, f32)) -> (f32, f32) {
        let gap = self.edge_gap();
        let along = |screen: f32, length: f32, start: bool, end: bool| {
            if start {
                gap
            } else if end {
                (screen - length - gap).max(0.0)
            } else {
                ((screen - length) * 0.5).max(0.0)
            }
        };
        (
            along(
                viewport.0,
                size.0,
                self.position == SheetPosition::Left,
                self.position == SheetPosition::Right,
            ),
            along(
                viewport.1,
                size.1,
                self.position == SheetPosition::Top,
                self.position == SheetPosition::Bottom,
            ),
        )
    }

    /// Fills the screen and puts `panel` where the window belongs.
    pub fn place<'a, Message: 'a>(
        &self,
        panel: impl Into<Element<'a, Message>>,
    ) -> Element<'a, Message> {
        self.align(container(panel)).into()
    }

    /// Makes `outer` fill the screen and puts its content where the window
    /// belongs (for a scrim that holds the panel).
    pub fn align<'a, Message: 'a>(&self, outer: Container<'a, Message>) -> Container<'a, Message> {
        outer
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(self.align_x())
            .align_y(self.align_y())
            .padding(self.edge_gap())
    }
}

fn fit_share(share: f32, screen: f32, auto: f32, floor: f32) -> f32 {
    if share > 0.0 { (screen * share).min(screen - SCREEN_MARGIN).max(floor) } else { auto }
}
