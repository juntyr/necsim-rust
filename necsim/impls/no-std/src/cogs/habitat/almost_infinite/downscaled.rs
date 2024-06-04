use core::num::NonZeroU32;

use necsim_core::{
    cogs::{Habitat, MathsCore, RngCore, UniformlySampleableHabitat},
    landscape::{IndexedLocation, LandscapeExtent, Location},
};
use necsim_core_bond::{OffByOneU32, OffByOneU64};

use super::AlmostInfiniteHabitat;

const ALMOST_INFINITE_EXTENT: LandscapeExtent =
    LandscapeExtent::new(Location::new(0, 0), OffByOneU32::max(), OffByOneU32::max());

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::lend::LendRustToCuda))]
#[cfg_attr(feature = "cuda", cuda(free = "M"))]
pub struct AlmostInfiniteDownscaledHabitat<M: MathsCore> {
    #[cfg_attr(feature = "cuda", cuda(embed))]
    habitat: AlmostInfiniteHabitat<M>,
    downscale_x: Log2U16,
    downscale_y: Log2U16,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, TypeLayout)]
#[repr(u16)]
pub enum Log2U16 {
    Shl0 = 1 << 0,
    Shl1 = 1 << 1,
    Shl2 = 1 << 2,
    Shl3 = 1 << 3,
    Shl4 = 1 << 4,
    Shl5 = 1 << 5,
    Shl6 = 1 << 6,
    Shl7 = 1 << 7,
    Shl8 = 1 << 8,
    Shl9 = 1 << 9,
    Shl10 = 1 << 10,
    Shl11 = 1 << 11,
    Shl12 = 1 << 12,
    Shl13 = 1 << 13,
    Shl14 = 1 << 14,
    Shl15 = 1 << 15,
}

impl<M: MathsCore> Clone for AlmostInfiniteDownscaledHabitat<M> {
    fn clone(&self) -> Self {
        Self {
            habitat: self.habitat.clone(),
            downscale_x: self.downscale_x,
            downscale_y: self.downscale_y,
        }
    }
}

impl<M: MathsCore> AlmostInfiniteDownscaledHabitat<M> {
    #[must_use]
    pub fn new(downscale_x: Log2U16, downscale_y: Log2U16) -> Self {
        Self {
            habitat: AlmostInfiniteHabitat::default(),
            downscale_x,
            downscale_y,
        }
    }

    #[must_use]
    pub fn downscale_x(&self) -> Log2U16 {
        self.downscale_x
    }

    #[must_use]
    pub fn downscale_y(&self) -> Log2U16 {
        self.downscale_y
    }

    #[must_use]
    pub fn downscale_area(&self) -> NonZeroU32 {
        // 2^{0..15} * 2^{0..15} >=1 and < 2^32
        unsafe { NonZeroU32::new_unchecked((self.downscale_x as u32) * (self.downscale_y as u32)) }
    }

    #[must_use]
    pub fn unscaled(&self) -> &AlmostInfiniteHabitat<M> {
        &self.habitat
    }
}

#[contract_trait]
impl<M: MathsCore> Habitat<M> for AlmostInfiniteDownscaledHabitat<M> {
    type LocationIterator<'a> = impl Iterator<Item = Location>;

    #[must_use]
    fn is_finite(&self) -> bool {
        false
    }

    #[must_use]
    fn get_extent(&self) -> &LandscapeExtent {
        &ALMOST_INFINITE_EXTENT
    }

    #[must_use]
    fn get_total_habitat(&self) -> OffByOneU64 {
        OffByOneU64::max()
    }

    #[must_use]
    fn get_habitat_at_location(&self, location: &Location) -> u32 {
        // TODO: optimise
        if ((location.x() % (self.downscale_x as u32)) == 0)
            && ((location.y() % (self.downscale_y as u32)) == 0)
        {
            (self.downscale_x as u32) * (self.downscale_y as u32)
        } else {
            0
        }
    }

    #[must_use]
    fn map_indexed_location_to_u64_injective(&self, indexed_location: &IndexedLocation) -> u64 {
        // TODO: optimise
        let index_x = indexed_location.index() % (self.downscale_x as u32);
        let index_y = indexed_location.index() / (self.downscale_x as u32);

        self.habitat
            .map_indexed_location_to_u64_injective(&IndexedLocation::new(
                Location::new(
                    indexed_location.location().x() + index_x,
                    indexed_location.location().y() + index_y,
                ),
                0,
            ))
    }

    #[must_use]
    fn iter_habitable_locations(&self) -> Self::LocationIterator<'_> {
        // TODO: optimise
        let width = unsafe {
            OffByOneU32::new_unchecked(OffByOneU32::max().get() / (self.downscale_x as u64))
        };
        let height = unsafe {
            OffByOneU32::new_unchecked(OffByOneU32::max().get() / (self.downscale_y as u64))
        };

        LandscapeExtent::new(Location::new(0, 0), width, height)
            .iter()
            .map(|location| {
                Location::new(
                    location.x() * (self.downscale_x as u32),
                    location.y() * (self.downscale_y as u32),
                )
            })
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M>> UniformlySampleableHabitat<M, G>
    for AlmostInfiniteDownscaledHabitat<M>
{
    #[must_use]
    #[inline]
    fn sample_habitable_indexed_location(&self, rng: &mut G) -> IndexedLocation {
        // TODO: optimise
        let location = self.habitat.sample_habitable_indexed_location(rng);

        let index_x = location.location().x() % (self.downscale_x as u32);
        let index_y = location.location().y() % (self.downscale_y as u32);

        IndexedLocation::new(
            Location::new(
                location.location().x() - index_x,
                location.location().y() - index_y,
            ),
            index_y * (self.downscale_x as u32) + index_x,
        )
    }
}
