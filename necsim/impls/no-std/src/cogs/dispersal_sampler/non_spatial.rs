use core::marker::PhantomData;

use necsim_core::{
    cogs::{Backup, DispersalSampler, Habitat, RngCore, SeparableDispersalSampler},
    landscape::Location,
};

use crate::cogs::habitat::non_spatial::NonSpatialHabitat;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
pub struct NonSpatialDispersalSampler<G: RngCore> {
    marker: PhantomData<G>,
}

impl<G: RngCore> Default for NonSpatialDispersalSampler<G> {
    #[must_use]
    fn default() -> Self {
        Self {
            marker: PhantomData::<G>,
        }
    }
}

#[contract_trait]
impl<G: RngCore> Backup for NonSpatialDispersalSampler<G> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            marker: PhantomData::<G>,
        }
    }
}

#[contract_trait]
impl<G: RngCore> DispersalSampler<NonSpatialHabitat, G> for NonSpatialDispersalSampler<G> {
    #[must_use]
    fn sample_dispersal_from_location(
        &self,
        _location: &Location,
        habitat: &NonSpatialHabitat,
        rng: &mut G,
    ) -> Location {
        use necsim_core::cogs::RngSampler;

        let habitat_index_max =
            (habitat.get_extent().width() as usize) * (habitat.get_extent().height() as usize);

        let dispersal_target_index = rng.sample_index(habitat_index_max);

        #[allow(clippy::cast_possible_truncation)]
        Location::new(
            (dispersal_target_index % (habitat.get_extent().width() as usize)) as u32
                + habitat.get_extent().x(),
            (dispersal_target_index / (habitat.get_extent().width() as usize)) as u32
                + habitat.get_extent().y(),
        )
    }
}

#[contract_trait]
impl<G: RngCore> SeparableDispersalSampler<NonSpatialHabitat, G> for NonSpatialDispersalSampler<G> {
    #[must_use]
    #[debug_requires((
        u64::from(habitat.get_extent().width()) * u64::from(habitat.get_extent().height())
    ) > 1_u64, "a different, non-self dispersal, target location exists")]
    fn sample_non_self_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &NonSpatialHabitat,
        rng: &mut G,
    ) -> Location {
        use necsim_core::cogs::RngSampler;

        let habitat_index_max =
            (habitat.get_extent().width() as usize) * (habitat.get_extent().height() as usize);
        let current_location_index = (location.y() as usize)
            * (habitat.get_extent().width() as usize)
            + (location.x() as usize);

        let dispersal_target_index = {
            let dispersal_target_index = rng.sample_index(habitat_index_max - 1);

            if dispersal_target_index >= current_location_index {
                dispersal_target_index + 1
            } else {
                dispersal_target_index
            }
        };

        #[allow(clippy::cast_possible_truncation)]
        Location::new(
            (dispersal_target_index % (habitat.get_extent().width() as usize)) as u32
                + habitat.get_extent().x(),
            (dispersal_target_index / (habitat.get_extent().width() as usize)) as u32
                + habitat.get_extent().y(),
        )
    }

    #[must_use]
    fn get_self_dispersal_probability_at_location(
        &self,
        _location: &Location,
        habitat: &NonSpatialHabitat,
    ) -> f64 {
        1.0_f64
            / (f64::from(habitat.get_extent().width()) * f64::from(habitat.get_extent().height()))
    }
}
