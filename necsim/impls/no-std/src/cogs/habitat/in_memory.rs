use core::marker::PhantomData;

use alloc::{boxed::Box, vec::Vec};

use r#final::Final;

use necsim_core::{
    cogs::{Backup, F64Core, Habitat},
    landscape::{IndexedLocation, LandscapeExtent, Location},
};

use crate::array2d::Array2D;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
pub struct InMemoryHabitat<F: F64Core> {
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    habitat: Final<Box<[u32]>>,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    u64_injection: Final<Box<[u64]>>,
    extent: LandscapeExtent,
    marker: PhantomData<F>,
}

#[contract_trait]
impl<F: F64Core> Backup for InMemoryHabitat<F> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            habitat: Final::new(self.habitat.clone()),
            u64_injection: Final::new(self.u64_injection.clone()),
            extent: self.extent.clone(),
            marker: PhantomData::<F>,
        }
    }
}

#[contract_trait]
impl<F: F64Core> Habitat<F> for InMemoryHabitat<F> {
    #[must_use]
    fn get_extent(&self) -> &LandscapeExtent {
        &self.extent
    }

    #[must_use]
    fn get_total_habitat(&self) -> u64 {
        self.habitat.iter().map(|x| u64::from(*x)).sum()
    }

    #[must_use]
    fn get_habitat_at_location(&self, location: &Location) -> u32 {
        self.habitat
            .get((location.y() as usize) * (self.extent.width() as usize) + (location.x() as usize))
            .copied()
            .unwrap_or(0)
    }

    #[must_use]
    fn map_indexed_location_to_u64_injective(&self, indexed_location: &IndexedLocation) -> u64 {
        self.u64_injection
            .get(
                (indexed_location.location().y() as usize) * (self.extent.width() as usize)
                    + (indexed_location.location().x() as usize),
            )
            .copied()
            .unwrap_or(0)
            + u64::from(indexed_location.index())
    }
}

impl<F: F64Core> InMemoryHabitat<F> {
    #[must_use]
    #[debug_ensures(
        old(habitat.num_columns()) == ret.get_extent().width() as usize &&
        old(habitat.num_rows()) == ret.get_extent().height() as usize,
        "habitat extent has the dimension of the habitat array"
    )]
    pub fn new(habitat: Array2D<u32>) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        let width: u32 = habitat.num_columns() as u32;
        #[allow(clippy::cast_possible_truncation)]
        let height: u32 = habitat.num_rows() as u32;

        let habitat = habitat.into_row_major().into_boxed_slice();

        let mut index_acc = 0_u64;

        let u64_injection = habitat
            .iter()
            .map(|h| {
                let injection = index_acc;
                index_acc += u64::from(*h);
                injection
            })
            .collect::<Vec<u64>>()
            .into_boxed_slice();

        #[allow(clippy::cast_possible_truncation)]
        let extent = LandscapeExtent::new(0, 0, width, height);

        Self {
            habitat: Final::new(habitat),
            u64_injection: Final::new(u64_injection),
            extent,
            marker: PhantomData::<F>,
        }
    }
}
