use array2d::Array2D;

use necsim_core::landscape::{LandscapeExtent, Location};

use super::Habitat;

#[allow(clippy::module_name_repetitions)]
pub struct InMemoryHabitat {
    habitat: Array2D<u32>,
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
    fn get_total_habitat(&self) -> usize {
        self.habitat
            .elements_row_major_iter()
            .map(|x| *x as usize)
            .sum()
    }

    #[must_use]
    fn get_habitat_at_location(&self, location: &Location) -> u32 {
        self.habitat[(location.y() as usize, location.x() as usize)]
    }
}

impl InMemoryHabitat {
    #[must_use]
    #[debug_ensures(
        old(habitat.num_columns()) == ret.get_extent().width() as usize &&
        old(habitat.num_rows()) == ret.get_extent().height() as usize
    )]
    pub fn new(habitat: Array2D<u32>) -> Self {
        Self { habitat }
    }
}
