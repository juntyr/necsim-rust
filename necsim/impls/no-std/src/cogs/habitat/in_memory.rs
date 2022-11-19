use core::marker::PhantomData;

use alloc::{boxed::Box, vec::Vec};

use r#final::Final;

use necsim_core::{
    cogs::{
        distribution::{IndexU64, Length},
        Backup, Distribution, Habitat, MathsCore, Rng, Samples, UniformlySampleableHabitat,
    },
    landscape::{IndexedLocation, LandscapeExtent, Location},
};
use necsim_core_bond::{OffByOneU32, OffByOneU64};

use crate::array2d::Array2D;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", cuda(free = "M"))]
#[cfg_attr(feature = "cuda", cuda(async = false))]
pub struct InMemoryHabitat<M: MathsCore> {
    #[cfg_attr(feature = "cuda", cuda(embed))]
    habitat: Final<Box<[u32]>>,
    #[cfg_attr(feature = "cuda", cuda(embed))]
    u64_injection: Final<Box<[u64]>>,
    extent: LandscapeExtent,
    marker: PhantomData<M>,
}

#[contract_trait]
impl<M: MathsCore> Backup for InMemoryHabitat<M> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            habitat: Final::new(self.habitat.clone()),
            u64_injection: Final::new(self.u64_injection.clone()),
            extent: self.extent.clone(),
            marker: PhantomData::<M>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore> Habitat<M> for InMemoryHabitat<M> {
    type LocationIterator<'a> = impl Iterator<Item = Location> + 'a;

    #[must_use]
    fn is_finite(&self) -> bool {
        true
    }

    #[must_use]
    fn get_extent(&self) -> &LandscapeExtent {
        &self.extent
    }

    #[must_use]
    fn get_total_habitat(&self) -> OffByOneU64 {
        // Safety: constructor ensures that there is at least one habitable location
        unsafe {
            OffByOneU64::new_unchecked(u128::from(*self.u64_injection.last().unwrap_unchecked()))
        }
    }

    #[must_use]
    fn get_habitat_at_location(&self, location: &Location) -> u32 {
        self.habitat
            .get(
                (location.y() as usize) * usize::from(self.extent.width())
                    + (location.x() as usize),
            )
            .copied()
            .unwrap_or(0)
    }

    #[must_use]
    fn map_indexed_location_to_u64_injective(&self, indexed_location: &IndexedLocation) -> u64 {
        self.u64_injection
            .get(
                (indexed_location.location().y() as usize) * usize::from(self.extent.width())
                    + (indexed_location.location().x() as usize),
            )
            .copied()
            .unwrap_or(0)
            + u64::from(indexed_location.index())
    }

    #[must_use]
    fn iter_habitable_locations(&self) -> Self::LocationIterator<'_> {
        self.habitat
            .iter()
            .enumerate()
            .filter_map(move |(location_index, habitat)| {
                if *habitat > 0 {
                    #[allow(clippy::cast_possible_truncation)]
                    Some(Location::new(
                        self.extent.x().wrapping_add(
                            (location_index % usize::from(self.extent.width())) as u32,
                        ),
                        self.extent.y().wrapping_add(
                            (location_index / usize::from(self.extent.width())) as u32,
                        ),
                    ))
                } else {
                    None
                }
            })
    }
}

#[contract_trait]
impl<M: MathsCore, G: Rng<M> + Samples<M, IndexU64>> UniformlySampleableHabitat<M, G>
    for InMemoryHabitat<M>
{
    #[must_use]
    #[inline]
    fn sample_habitable_indexed_location(&self, rng: &mut G) -> IndexedLocation {
        let indexed_location_index =
            IndexU64::sample_with(rng, Length(self.get_total_habitat().into()));

        let location_index = match self.u64_injection.binary_search(&indexed_location_index) {
            Ok(index) => index,
            Err(index) => index - 1,
        };

        #[allow(clippy::cast_possible_truncation)]
        IndexedLocation::new(
            Location::new(
                self.extent
                    .x()
                    .wrapping_add((location_index % usize::from(self.extent.width())) as u32),
                self.extent
                    .y()
                    .wrapping_add((location_index / usize::from(self.extent.width())) as u32),
            ),
            (indexed_location_index - self.u64_injection[location_index]) as u32,
        )
    }
}

impl<M: MathsCore> InMemoryHabitat<M> {
    #[must_use]
    #[debug_ensures(if let Some(ret) = &ret {
        old(habitat.num_columns()) == usize::from(ret.get_extent().width()) &&
        old(habitat.num_rows()) == usize::from(ret.get_extent().height())
    } else { true }, "habitat extent has the dimension of the habitat array")]
    pub fn try_new(habitat: Array2D<u32>) -> Option<Self> {
        let Ok(width) = OffByOneU32::new(habitat.num_columns() as u64) else {
            return None
        };
        let Ok(height) = OffByOneU32::new(habitat.num_rows() as u64) else {
            return None
        };

        let habitat = habitat.into_row_major().into_boxed_slice();

        let mut index_acc = 0_u64;

        let mut u64_injection = habitat
            .iter()
            .map(|h| {
                let injection = index_acc;
                index_acc += u64::from(*h);
                injection
            })
            .collect::<Vec<u64>>();
        u64_injection.push(index_acc);
        let u64_injection = u64_injection.into_boxed_slice();

        if index_acc == 0 {
            return None;
        }

        #[allow(clippy::cast_possible_truncation)]
        let extent = LandscapeExtent::new(0, 0, width, height);

        Some(Self {
            habitat: Final::new(habitat),
            u64_injection: Final::new(u64_injection),
            extent,
            marker: PhantomData::<M>,
        })
    }
}
