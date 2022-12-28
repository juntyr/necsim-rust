use serde::{Deserialize, Serialize};

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
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn x(&self) -> u32 {
        self.x
    }

    #[must_use]
    pub const fn y(&self) -> u32 {
        self.y
    }
}

impl From<IndexedLocation> for Location {
    fn from(indexed_location: IndexedLocation) -> Location {
        indexed_location.location
    }
}

#[derive(
    Eq, PartialEq, PartialOrd, Ord, Clone, Hash, Debug, Serialize, Deserialize, TypeLayout,
)]
#[allow(clippy::module_name_repetitions, clippy::unsafe_derive_deserialize)]
#[serde(from = "IndexedLocationRaw", into = "IndexedLocationRaw")]
#[repr(C)]
pub struct IndexedLocation {
    location: Location,
    index: u32,
}

impl IndexedLocation {
    #[must_use]
    pub const fn new(location: Location, index: u32) -> Self {
        Self { location, index }
    }

    #[must_use]
    pub const fn location(&self) -> &Location {
        &self.location
    }

    #[must_use]
    pub const fn index(&self) -> u32 {
        self.index
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
    index: u32,
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
