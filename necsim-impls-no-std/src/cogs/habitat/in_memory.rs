use array2d::Array2D;

use necsim_core::{
    cogs::{Habitat, HabitatToU64Injection},
    landscape::{IndexedLocation, LandscapeExtent, Location},
};

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[derive(Debug)]
pub struct InMemoryHabitat {
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    habitat: Array2D<u32>,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    u64_injection: Array2D<u64>,
}

#[contract_trait]
impl Habitat for InMemoryHabitat {
    #[must_use]
    fn get_extent(&self) -> LandscapeExtent {
        #[allow(clippy::cast_possible_truncation)]
        LandscapeExtent::new(
            0,
            0,
            self.habitat.num_columns() as u32,
            self.habitat.num_rows() as u32,
        )
    }

    #[must_use]
    fn get_total_habitat(&self) -> u64 {
        self.habitat
            .elements_row_major_iter()
            .map(|x| *x as u64)
            .sum()
    }

    #[must_use]
    fn get_habitat_at_location(&self, location: &Location) -> u32 {
        self.habitat[(location.y() as usize, location.x() as usize)]
    }
}

#[contract_trait]
impl HabitatToU64Injection for InMemoryHabitat {
    #[must_use]
    fn map_indexed_location_to_u64_injective(&self, indexed_location: &IndexedLocation) -> u64 {
        self.u64_injection[(
            indexed_location.location().y() as usize,
            indexed_location.location().x() as usize,
        )] + u64::from(indexed_location.index())
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
        let mut index_acc = 0_u64;

        let u64_injection = Array2D::from_iter_row_major(
            habitat.elements_row_major_iter().map(|h| {
                let injection = index_acc;
                index_acc += u64::from(*h);
                injection
            }),
            habitat.num_rows(),
            habitat.num_columns(),
        )
        .unwrap();

        Self {
            habitat,
            u64_injection,
        }
    }
}
