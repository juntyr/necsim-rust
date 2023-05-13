use core::{marker::PhantomData, num::NonZeroU64};

use necsim_core::{
    cogs::{
        distribution::{IndexU64, Length},
        Backup, DispersalSampler, Distribution, Habitat, MathsCore, Rng, Samples,
        SeparableDispersalSampler,
    },
    landscape::Location,
};
use necsim_core_bond::ClosedUnitF64;

use crate::cogs::habitat::non_spatial::NonSpatialHabitat;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", cuda(free = "M", free = "G"))]
pub struct NonSpatialDispersalSampler<M: MathsCore, G: Rng<M> + Samples<M, IndexU64>> {
    marker: PhantomData<(M, G)>,
}

impl<M: MathsCore, G: Rng<M> + Samples<M, IndexU64>> Default for NonSpatialDispersalSampler<M, G> {
    #[must_use]
    fn default() -> Self {
        Self {
            marker: PhantomData::<(M, G)>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore, G: Rng<M> + Samples<M, IndexU64>> Backup for NonSpatialDispersalSampler<M, G> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            marker: PhantomData::<(M, G)>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore, G: Rng<M> + Samples<M, IndexU64>> DispersalSampler<M, NonSpatialHabitat<M>, G>
    for NonSpatialDispersalSampler<M, G>
{
    #[must_use]
    #[inline]
    fn sample_dispersal_from_location(
        &self,
        _location: &Location,
        habitat: &NonSpatialHabitat<M>,
        rng: &mut G,
    ) -> Location {
        let habitat_index_max =
            habitat.get_extent().width().get() * habitat.get_extent().height().get();

        // Safety: habitat width and height are both > 0
        let dispersal_target_index = IndexU64::sample_with(
            rng,
            Length(unsafe { NonZeroU64::new_unchecked(habitat_index_max) }),
        );

        #[allow(clippy::cast_possible_truncation)]
        Location::new(
            habitat
                .get_extent()
                .x()
                .wrapping_add((dispersal_target_index % habitat.get_extent().width().get()) as u32),
            habitat
                .get_extent()
                .y()
                .wrapping_add((dispersal_target_index / habitat.get_extent().width().get()) as u32),
        )
    }
}

#[contract_trait]
impl<M: MathsCore, G: Rng<M> + Samples<M, IndexU64>>
    SeparableDispersalSampler<M, NonSpatialHabitat<M>, G> for NonSpatialDispersalSampler<M, G>
{
    #[must_use]
    #[debug_requires((
        u64::from(habitat.get_extent().width()) * u64::from(habitat.get_extent().height())
    ) > 1_u64, "a different, non-self dispersal, target location exists")]
    fn sample_non_self_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &NonSpatialHabitat<M>,
        rng: &mut G,
    ) -> Location {
        let habitat_index_max =
            habitat.get_extent().width().get() * habitat.get_extent().height().get();
        let current_location_index =
            u64::from(location.y()) * habitat.get_extent().width().get() + u64::from(location.x());

        let dispersal_target_index = {
            // Safety: by PRE, `habitat_index_max` > 1
            let dispersal_target_index = IndexU64::sample_with(
                rng,
                Length(unsafe { NonZeroU64::new_unchecked(habitat_index_max - 1) }),
            );

            if dispersal_target_index >= current_location_index {
                dispersal_target_index + 1
            } else {
                dispersal_target_index
            }
        };

        #[allow(clippy::cast_possible_truncation)]
        Location::new(
            habitat
                .get_extent()
                .x()
                .wrapping_add((dispersal_target_index % habitat.get_extent().width().get()) as u32),
            habitat
                .get_extent()
                .y()
                .wrapping_add((dispersal_target_index / habitat.get_extent().width().get()) as u32),
        )
    }

    #[must_use]
    fn get_self_dispersal_probability_at_location(
        &self,
        _location: &Location,
        habitat: &NonSpatialHabitat<M>,
    ) -> ClosedUnitF64 {
        let self_dispersal = 1.0_f64
            / (f64::from(habitat.get_extent().width()) * f64::from(habitat.get_extent().height()));

        // Safety: Since the method is only called for a valid location,
        //          width >= 1 and height >= 1
        //         => 1.0/(width*height) in [0.0; 1.0]
        unsafe { ClosedUnitF64::new_unchecked(self_dispersal) }
    }
}
