use array2d::Array2D;

use alloc::{boxed::Box, vec::Vec};

use necsim_core::{
    cogs::Habitat,
    landscape::{IndexedLocation, LandscapeExtent, Location},
};

use crate::cogs::origin_sampler::in_memory::InMemoryOriginSampler;

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[derive(Debug)]
pub struct InMemoryHabitat {
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    habitat: Box<[u32]>,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    u64_injection: Box<[u64]>,
    extent: LandscapeExtent,
}

#[contract_trait]
impl Habitat for InMemoryHabitat {
    type OriginSampler<'h> = InMemoryOriginSampler<'h>;

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
            [(location.y() as usize) * (self.extent.width() as usize) + (location.x() as usize)]
    }

    #[must_use]
    fn map_indexed_location_to_u64_injective(&self, indexed_location: &IndexedLocation) -> u64 {
        self.u64_injection[(indexed_location.location().y() as usize)
            * (self.extent.width() as usize)
            + (indexed_location.location().x() as usize)]
            + u64::from(indexed_location.index())
    }
}

impl InMemoryHabitat {
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
            habitat,
            u64_injection,
            extent,
        }
    }
}
