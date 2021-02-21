use core::num::NonZeroU32;

use necsim_core::{cogs::Habitat, landscape::Location};

use crate::decomposition::Decomposition;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ModuloDecomposition {
    offset: u32,
    stride: NonZeroU32,
}

impl ModuloDecomposition {
    #[must_use]
    pub fn new(offset: u32, stride: NonZeroU32) -> Self {
        Self { offset, stride }
    }
}

#[contract_trait]
impl<H: Habitat> Decomposition<H> for ModuloDecomposition {
    fn get_subdomain_rank(&self) -> u32 {
        self.offset
    }

    fn get_number_of_subdomains(&self) -> NonZeroU32 {
        self.stride
    }

    fn map_location_to_subdomain_rank(&self, location: &Location, habitat: &H) -> u32 {
        let extent = habitat.get_extent();

        let location_index = u64::from(location.y() - extent.y()) * u64::from(extent.width())
            + u64::from(location.x() - extent.x());

        #[allow(clippy::cast_possible_truncation)]
        {
            (location_index % u64::from(self.stride.get())) as u32
        }
    }
}
