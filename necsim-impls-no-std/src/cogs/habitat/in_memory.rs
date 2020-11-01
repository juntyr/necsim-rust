use alloc::boxed::Box;

use necsim_core::cogs::Habitat;
use necsim_core::landscape::{LandscapeExtent, Location};

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda, LendToCuda))]
pub struct InMemoryHabitat {
    #[cfg_attr(feature = "cuda", r2c)]
    habitat: Box<[u32]>,
    width: u32,
    height: u32,
}

/*#[cfg_attr(feature = "cuda", derive(RustToCuda, LendToCuda))]
pub struct Simulation<H: Habitat + crate::shim::cuda::RustToCuda> {
    #[cfg_attr(feature = "cuda", r2c)]
    habitat: H,
}*/

#[contract_trait]
impl Habitat for InMemoryHabitat {
    #[must_use]
    fn get_extent(&self) -> LandscapeExtent {
        #[allow(clippy::cast_possible_truncation)]
        LandscapeExtent::new(0, 0, self.width, self.height)
    }

    #[must_use]
    fn get_total_habitat(&self) -> usize {
        self.habitat.iter().map(|x| *x as usize).sum()
    }

    #[must_use]
    fn get_habitat_at_location(&self, location: &Location) -> u32 {
        let location_index =
            (location.y() as usize) * (self.width as usize) + (location.x() as usize);

        self.habitat[location_index]
    }
}

impl InMemoryHabitat {
    #[must_use]
    #[debug_requires(
        habitat.len() == (width as usize) * (height as usize),
        "habitat contains a 2d array of exact size width x height"
    )]
    #[debug_ensures(
        old(width) == ret.get_extent().width() &&
        old(height) == ret.get_extent().height(),
        "habitat extent has the dimension of the habitat array"
    )]
    pub fn new(habitat: Box<[u32]>, width: u32, height: u32) -> Self {
        #[allow(clippy::cast_possible_truncation)]
        Self {
            habitat,
            width,
            height,
        }
    }
}
