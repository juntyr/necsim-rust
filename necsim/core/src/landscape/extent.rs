use necsim_core_bond::OffByOneU32;

use super::Location;

#[allow(clippy::module_name_repetitions, clippy::unsafe_derive_deserialize)]
#[derive(PartialEq, Eq, Clone, Debug, serde::Deserialize, serde::Serialize, TypeLayout)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::lend::LendRustToCuda))]
#[repr(C)]
#[cfg_attr(feature = "cuda", cuda(ignore))]
#[serde(rename = "Extent")]
#[serde(deny_unknown_fields)]
pub struct LandscapeExtent {
    x: u32,
    y: u32,
    width: OffByOneU32,
    height: OffByOneU32,
}

impl LandscapeExtent {
    #[must_use]
    pub const fn new(x: u32, y: u32, width: OffByOneU32, height: OffByOneU32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    #[must_use]
    pub const fn x(&self) -> u32 {
        self.x
    }

    #[must_use]
    pub const fn y(&self) -> u32 {
        self.y
    }

    #[must_use]
    pub const fn width(&self) -> OffByOneU32 {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> OffByOneU32 {
        self.height
    }

    #[must_use]
    pub const fn contains(&self, location: &Location) -> bool {
        location.x() >= self.x
            && location.x() <= self.width.add_incl(self.x)
            && location.y() >= self.y
            && location.y() <= self.height.add_incl(self.y)
    }

    #[must_use]
    pub fn iter(&self) -> LocationIterator {
        LocationIterator {
            x: self.x,
            y: self.y,
            extent: self.clone(),
            first_y: true,
        }
    }
}

impl IntoIterator for &LandscapeExtent {
    type IntoIter = LocationIterator;
    type Item = Location;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LocationIterator {
    x: u32,
    y: u32,
    extent: LandscapeExtent,
    first_y: bool,
}

impl Iterator for LocationIterator {
    type Item = Location;

    fn next(&mut self) -> Option<Self::Item> {
        if self.y != self.extent.height().add_excl(self.extent.y()) || self.first_y {
            let next = Some(Location::new(self.x, self.y));

            self.x = if self.x == self.extent.width().add_incl(self.extent.x()) {
                self.y = self.y.wrapping_add(1);
                self.first_y = false;
                self.extent.x()
            } else {
                self.x.wrapping_add(1)
            };

            next
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use super::{LandscapeExtent, Location, LocationIterator, OffByOneU32};

    const M1: u32 = 0_u32.wrapping_sub(1);
    const M2: u32 = 0_u32.wrapping_sub(2);

    #[test]
    fn test_single_location() {
        let extent = LandscapeExtent::new(
            0,
            0,
            OffByOneU32::new(1).unwrap(),
            OffByOneU32::new(1).unwrap(),
        );
        let locations: Vec<Location> = extent.iter().collect();
        assert_eq!(locations, vec![Location::new(0, 0)]);
    }

    #[test]
    fn test_simple_extent() {
        let extent = LandscapeExtent::new(
            42,
            24,
            OffByOneU32::new(4).unwrap(),
            OffByOneU32::new(2).unwrap(),
        );
        let locations: Vec<Location> = extent.iter().collect();
        assert_eq!(
            locations,
            vec![
                Location::new(42, 24),
                Location::new(43, 24),
                Location::new(44, 24),
                Location::new(45, 24),
                Location::new(42, 25),
                Location::new(43, 25),
                Location::new(44, 25),
                Location::new(45, 25)
            ]
        );
    }

    #[test]
    fn test_wrapping_extent() {
        let extent = LandscapeExtent::new(
            M2,
            M1,
            OffByOneU32::new(4).unwrap(),
            OffByOneU32::new(2).unwrap(),
        );
        let locations: Vec<Location> = extent.iter().collect();
        assert_eq!(
            locations,
            vec![
                Location::new(M2, M1),
                Location::new(M1, M1),
                Location::new(0, M1),
                Location::new(1, M1),
                Location::new(M2, 0),
                Location::new(M1, 0),
                Location::new(0, 0),
                Location::new(1, 0)
            ]
        );
    }

    #[test]
    fn test_full_extent() {
        let extent = LandscapeExtent::new(
            0,
            0,
            OffByOneU32::new(1 << 32).unwrap(),
            OffByOneU32::new(1 << 32).unwrap(),
        );
        let mut iter = extent.iter();
        assert_eq!(
            iter,
            LocationIterator {
                x: 0,
                y: 0,
                extent: extent.clone(),
                first_y: true,
            }
        );
        assert_eq!(iter.next(), Some(Location::new(0, 0)));

        iter.x = M1;
        iter.y = M1;
        assert_eq!(iter.next(), Some(Location::new(M1, M1)));
        assert_eq!(
            iter,
            LocationIterator {
                x: 0,
                y: 0,
                extent: extent.clone(),
                first_y: false,
            }
        );
        assert_eq!(iter.next(), None);
        assert_eq!(
            iter,
            LocationIterator {
                x: 0,
                y: 0,
                extent,
                first_y: false,
            }
        );
    }

    #[test]
    fn test_full_wrapping_extent() {
        let extent = LandscapeExtent::new(
            1386,
            6812,
            OffByOneU32::new(1 << 32).unwrap(),
            OffByOneU32::new(1 << 32).unwrap(),
        );
        let mut iter = extent.iter();
        assert_eq!(
            iter,
            LocationIterator {
                x: 1386,
                y: 6812,
                extent: extent.clone(),
                first_y: true,
            }
        );

        iter.x = M1;
        assert_eq!(iter.next(), Some(Location::new(M1, 6812)));
        assert_eq!(
            iter,
            LocationIterator {
                x: 0,
                y: 6812,
                extent: extent.clone(),
                first_y: true,
            }
        );
        assert_eq!(iter.next(), Some(Location::new(0, 6812)));

        iter.x = 1385;
        assert_eq!(iter.next(), Some(Location::new(1385, 6812)));
        assert_eq!(
            iter,
            LocationIterator {
                x: 1386,
                y: 6813,
                extent: extent.clone(),
                first_y: false,
            }
        );
        assert_eq!(iter.next(), Some(Location::new(1386, 6813)));

        iter.x = 1385;
        iter.y = M1;
        assert_eq!(iter.next(), Some(Location::new(1385, M1)));
        assert_eq!(
            iter,
            LocationIterator {
                x: 1386,
                y: 0,
                extent: extent.clone(),
                first_y: false,
            }
        );
        assert_eq!(iter.next(), Some(Location::new(1386, 0)));

        iter.x = 1385;
        iter.y = 6811;
        assert_eq!(iter.next(), Some(Location::new(1385, 6811)));
        assert_eq!(
            iter,
            LocationIterator {
                x: 1386,
                y: 6812,
                extent: extent.clone(),
                first_y: false,
            }
        );
        assert_eq!(iter.next(), None);
        assert_eq!(
            iter,
            LocationIterator {
                x: 1386,
                y: 6812,
                extent,
                first_y: false,
            }
        );
    }
}
