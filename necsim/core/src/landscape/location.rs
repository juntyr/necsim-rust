use core::num::NonZeroU32;

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

impl From<IndexedLocation> for Location {
    fn from(indexed_location: IndexedLocation) -> Location {
        indexed_location.location
    }
}

// IndexedLocation uses a NonZeroU32 index internally to enable same-size
//  Option optimisation

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
#[cfg_attr(feature = "cuda", derive(DeviceCopy))]
#[allow(clippy::module_name_repetitions)]
pub struct IndexedLocation {
    location: Location,
    pub(crate) index: NonZeroU32,
}

impl IndexedLocation {
    #[must_use]
    #[debug_ensures(
        ret.location.x() == old(location.x()) && ret.location.y() == old(location.y()),
        "stores location"
    )]
    #[debug_ensures(ret.index() == index, "stores index")]
    pub fn new(location: Location, index: u32) -> Self {
        Self {
            location,
            index: unsafe { NonZeroU32::new_unchecked(index + 1) },
        }
    }

    #[must_use]
    pub fn location(&self) -> &Location {
        &self.location
    }

    #[must_use]
    pub fn index(&self) -> u32 {
        self.index.get() - 1
    }
}
