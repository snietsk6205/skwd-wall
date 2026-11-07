#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Slices,
    Depth,
    Grid,
    Hex,
    Sandy,
    Hand,
    Collection,
    /// Custom build: 3D coverflow, carousel ring or receding stack.
    Coverflow,
    /// Custom build: one big preview with a filmstrip of the others.
    Showcase,
}

impl Mode {
    pub const ALL: [Self; 9] = [
        Self::Slices,
        Self::Depth,
        Self::Hex,
        Self::Grid,
        Self::Sandy,
        Self::Hand,
        Self::Collection,
        Self::Coverflow,
        Self::Showcase,
    ];

    pub fn try_from_key(value: &str) -> Option<Self> {
        match value {
            "slices" => Some(Self::Slices),
            "depth" => Some(Self::Depth),
            "wall" | "grid" => Some(Self::Grid),
            "hex" => Some(Self::Hex),
            "sandy" | "nova" => Some(Self::Sandy),
            "hand" => Some(Self::Hand),
            "collection" => Some(Self::Collection),
            "coverflow" | "carousel" => Some(Self::Coverflow),
            "showcase" | "filmstrip" => Some(Self::Showcase),
            _ => None,
        }
    }

    pub fn from_key(value: &str) -> Self {
        Self::try_from_key(value).unwrap_or(Self::Slices)
    }

    pub const fn as_key(self) -> &'static str {
        match self {
            Self::Slices => "slices",
            Self::Depth => "depth",
            Self::Grid => "wall",
            Self::Hex => "hex",
            Self::Sandy => "sandy",
            Self::Hand => "hand",
            Self::Collection => "collection",
            Self::Coverflow => "coverflow",
            Self::Showcase => "showcase",
        }
    }
}

mod tests;
