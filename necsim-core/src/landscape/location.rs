#[derive(Eq, PartialEq, Clone, Hash, Debug)]
#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
pub struct Location {
    x: u32,
    y: u32,
}

impl Location {
    #[must_use]
    #[debug_ensures(ret.x() == x, "stores x")]
    #[debug_ensures(ret.y() == y, "stores y")]
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn x(&self) -> u32 {
        self.x
    }

    #[must_use]
    pub fn y(&self) -> u32 {
        self.y
    }
}
