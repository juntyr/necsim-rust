use core::num::NonZeroU32;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::cogs::Backup;

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(
    Eq, PartialEq, PartialOrd, Ord, Clone, Hash, Debug, Serialize, Deserialize, TypeLayout,
)]
#[serde(deny_unknown_fields)]
#[repr(C)]
pub struct Location {
    x: u32,
    y: u32,
}

#[contract_trait]
impl Backup for Location {
    unsafe fn backup_unchecked(&self) -> Self {
        self.clone()
    }
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
#[derive(
    Eq, PartialEq, PartialOrd, Ord, Clone, Hash, Debug, Serialize, Deserialize, TypeLayout,
)]
#[allow(clippy::module_name_repetitions, clippy::unsafe_derive_deserialize)]
#[serde(from = "IndexedLocationRaw", into = "IndexedLocationRaw")]
#[repr(C)]
pub struct IndexedLocation {
    location: Location,
    index: LocationIndex,
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
            index: LocationIndex::new(index),
        }
    }

    #[must_use]
    pub fn location(&self) -> &Location {
        &self.location
    }

    #[must_use]
    pub fn index(&self) -> u32 {
        self.index.get()
    }
}

#[derive(Eq, PartialEq, PartialOrd, Ord, Clone, Copy, Hash, Debug, TypeLayout)]
#[repr(transparent)]
struct LocationIndex(NonZeroU32);

impl LocationIndex {
    fn new(index: u32) -> Self {
        Self(unsafe { NonZeroU32::new_unchecked(index + 1) })
    }

    fn get(self) -> u32 {
        self.0.get() - 1
    }
}

impl Serialize for LocationIndex {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (self.0.get() - 1).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for LocationIndex {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let inner = u32::deserialize(deserializer)?;

        Ok(Self(unsafe { NonZeroU32::new_unchecked(inner + 1) }))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename = "IndexedLocation")]
#[repr(C)]
struct IndexedLocationRaw {
    x: u32,
    y: u32,
    #[serde(alias = "i")]
    index: LocationIndex,
}

impl From<IndexedLocation> for IndexedLocationRaw {
    fn from(val: IndexedLocation) -> Self {
        Self {
            x: val.location.x,
            y: val.location.y,
            index: val.index,
        }
    }
}

impl From<IndexedLocationRaw> for IndexedLocation {
    fn from(raw: IndexedLocationRaw) -> Self {
        Self {
            location: Location::new(raw.x, raw.y),
            index: raw.index,
        }
    }
}
