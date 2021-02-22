use super::Location;

#[allow(clippy::module_name_repetitions)]
#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
pub struct LandscapeExtent {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl LandscapeExtent {
    #[must_use]
    #[debug_ensures(ret.x() == x, "stores x")]
    #[debug_ensures(ret.y() == y, "stores y")]
    #[debug_ensures(ret.width() == width, "stores width")]
    #[debug_ensures(ret.height() == height, "stores height")]
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

    #[must_use]
    pub fn contains(&self, location: &Location) -> bool {
        location.x() >= self.x
            && location.x() < (self.x + self.width)
            && location.y() >= self.y
            && location.y() < (self.y + self.height)
    }

    #[must_use]
    pub fn iter(&self) -> LocationIterator {
        LocationIterator {
            x: self.x,
            y: self.y,
            extent: self.clone(),
        }
    }
}

#[derive(Debug)]
pub struct LocationIterator {
    x: u32,
    y: u32,
    extent: LandscapeExtent,
}

impl Iterator for LocationIterator {
    type Item = Location;

    fn next(&mut self) -> Option<Self::Item> {
        if self.y < (self.extent.y() + self.extent.height()) {
            let next = Some(Location::new(self.x, self.y));

            self.x = if (self.x + 1) >= (self.extent.x() + self.extent.width()) {
                self.y += 1;
                self.extent.x()
            } else {
                self.x + 1
            };

            next
        } else {
            None
        }
    }
}
