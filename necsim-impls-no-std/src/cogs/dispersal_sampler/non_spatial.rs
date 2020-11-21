use core::marker::PhantomData;

use necsim_core::{
    cogs::{DispersalSampler, Habitat, RngCore, SeparableDispersalSampler},
    landscape::{LandscapeExtent, Location},
};

use crate::cogs::habitat::non_spatial::NonSpatialHabitat;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
pub struct NonSpatialDispersalSampler<G: RngCore> {
    habitat_extent: LandscapeExtent,
    marker: PhantomData<G>,
}

impl<G: RngCore> NonSpatialDispersalSampler<G> {
    #[must_use]
    pub fn new(habitat: &NonSpatialHabitat) -> Self {
        Self {
            habitat_extent: habitat.get_extent(),
            marker: PhantomData::<G>,
        }
    }
}

impl<G: RngCore> DispersalSampler<NonSpatialHabitat, G> for NonSpatialDispersalSampler<G> {
    #[must_use]
    #[debug_requires(self.habitat_extent.contains(location), "location is inside habitat extent")]
    #[debug_ensures(self.habitat_extent.contains(&ret), "target is inside habitat extent")]
    fn sample_dispersal_from_location(&self, location: &Location, rng: &mut G) -> Location {
        use necsim_core::cogs::RngSampler;

        let habitat_index_max =
            (self.habitat_extent.width() as usize) * (self.habitat_extent.height() as usize);

        let dispersal_target_index = rng.sample_index(habitat_index_max);

        #[allow(clippy::cast_possible_truncation)]
        Location::new(
            (dispersal_target_index % (self.habitat_extent.width() as usize)) as u32
                + self.habitat_extent.x(),
            (dispersal_target_index / (self.habitat_extent.width() as usize)) as u32
                + self.habitat_extent.y(),
        )
    }
}

#[contract_trait]
impl<G: RngCore> SeparableDispersalSampler<NonSpatialHabitat, G> for NonSpatialDispersalSampler<G> {
    #[must_use]
    #[debug_requires(self.habitat_extent.contains(location), "location is inside habitat extent")]
    #[debug_requires(
        (u64::from(self.habitat_extent.width()) * u64::from(self.habitat_extent.height())) > 1_u64,
        "a different, non-self dispersal, target location exists"
    )]
    #[debug_ensures(self.habitat_extent.contains(&ret), "target is inside habitat extent")]
    fn sample_non_self_dispersal_from_location(
        &self,
        location: &Location,
        rng: &mut G,
    ) -> Location {
        use necsim_core::cogs::RngSampler;

        let habitat_index_max =
            (self.habitat_extent.width() as usize) * (self.habitat_extent.height() as usize);
        let current_location_index = (location.y() as usize)
            * (self.habitat_extent.width() as usize)
            + (location.x() as usize);

        let dispersal_target_index = {
            let dispersal_target_index = rng.sample_index(habitat_index_max - 1);

            if dispersal_target_index == current_location_index {
                dispersal_target_index + 1
            } else {
                dispersal_target_index
            }
        };

        #[allow(clippy::cast_possible_truncation)]
        Location::new(
            (dispersal_target_index % (self.habitat_extent.width() as usize)) as u32
                + self.habitat_extent.x(),
            (dispersal_target_index / (self.habitat_extent.width() as usize)) as u32
                + self.habitat_extent.y(),
        )
    }

    #[must_use]
    #[debug_requires(self.habitat_extent.contains(location), "location is inside habitat extent")]
    fn get_self_dispersal_probability_at_location(&self, location: &Location) -> f64 {
        1.0_f64 / (f64::from(self.habitat_extent.width()) * f64::from(self.habitat_extent.height()))
    }
}
