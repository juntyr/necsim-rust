#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq)]
pub struct LandscapeExtent {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl LandscapeExtent {
    #[must_use]
    #[debug_ensures(
        ret.x() == x &&
        ret.y() == y &&
        ret.width() == width &&
        ret.height() == height
    )]
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub fn x(&self) -> u32 {
        self.x
    }

    #[must_use]
    pub fn y(&self) -> u32 {
        self.y
    }

    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }
}
