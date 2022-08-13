#![allow(clippy::trait_duplication_in_bounds)]

use necsim_core::{
    cogs::{
        distribution::{Bernoulli, IndexU64},
        Backup, DispersalSampler, Distribution, Habitat, MathsCore, Rng, Samples,
        SeparableDispersalSampler,
    },
    landscape::Location,
};
use necsim_core_bond::{ClosedUnitF64, OpenClosedUnitF64 as PositiveUnitF64};

use crate::cogs::{
    dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
    habitat::spatially_implicit::SpatiallyImplicitHabitat,
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", cuda(free = "M"))]
pub struct SpatiallyImplicitDispersalSampler<
    M: MathsCore,
    G: Rng<M> + Samples<M, IndexU64> + Samples<M, Bernoulli>,
> {
    #[cfg_attr(feature = "cuda", cuda(embed))]
    local: NonSpatialDispersalSampler<M, G>,
    #[cfg_attr(feature = "cuda", cuda(embed))]
    meta: NonSpatialDispersalSampler<M, G>,
    local_migration_probability_per_generation: PositiveUnitF64,
}

#[allow(clippy::trait_duplication_in_bounds)]
impl<M: MathsCore, G: Rng<M> + Samples<M, IndexU64> + Samples<M, Bernoulli>>
    SpatiallyImplicitDispersalSampler<M, G>
{
    #[must_use]
    pub fn new(local_migration_probability_per_generation: PositiveUnitF64) -> Self {
        Self {
            local: NonSpatialDispersalSampler::default(),
            meta: NonSpatialDispersalSampler::default(),
            local_migration_probability_per_generation,
        }
    }
}

#[allow(clippy::trait_duplication_in_bounds)]
#[contract_trait]
impl<M: MathsCore, G: Rng<M> + Samples<M, IndexU64> + Samples<M, Bernoulli>> Backup
    for SpatiallyImplicitDispersalSampler<M, G>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            local: self.local.backup_unchecked(),
            meta: self.meta.backup_unchecked(),
            local_migration_probability_per_generation: self
                .local_migration_probability_per_generation,
        }
    }
}

#[allow(clippy::trait_duplication_in_bounds)]
#[contract_trait]
impl<M: MathsCore, G: Rng<M> + Samples<M, IndexU64> + Samples<M, Bernoulli>>
    DispersalSampler<M, SpatiallyImplicitHabitat<M>, G>
    for SpatiallyImplicitDispersalSampler<M, G>
{
    #[must_use]
    #[debug_ensures(habitat.meta().get_extent().contains(&ret) || (
        if old(habitat.local().get_extent().contains(location)) {
            habitat.local().get_extent().contains(&ret)
        } else { false }
    ), "target is inside the meta habitat extent, \
        or -- iff the location was local -- in the local habitat extent"
    )]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &SpatiallyImplicitHabitat<M>,
        rng: &mut G,
    ) -> Location {
        // By PRE, location must be habitable, i.e. either in the local or the meta
        //  habitat
        if habitat.local().get_extent().contains(location) {
            if Bernoulli::sample_with(rng, self.local_migration_probability_per_generation.into()) {
                // Provide a dummpy Location in the meta community to disperse from
                self.meta.sample_dispersal_from_location(
                    &Location::new(
                        habitat.meta().get_extent().x(),
                        habitat.meta().get_extent().y(),
                    ),
                    habitat.meta(),
                    rng,
                )
            } else {
                self.local
                    .sample_dispersal_from_location(location, habitat.local(), rng)
            }
        } else {
            self.meta
                .sample_dispersal_from_location(location, habitat.meta(), rng)
        }
    }
}

#[allow(clippy::trait_duplication_in_bounds)]
#[contract_trait]
impl<M: MathsCore, G: Rng<M> + Samples<M, IndexU64> + Samples<M, Bernoulli>>
    SeparableDispersalSampler<M, SpatiallyImplicitHabitat<M>, G>
    for SpatiallyImplicitDispersalSampler<M, G>
{
    #[must_use]
    #[debug_ensures(habitat.meta().get_extent().contains(&ret) || (
        if old(habitat.local().get_extent().contains(location)) {
            habitat.local().get_extent().contains(&ret)
        } else { false }
    ), "target is inside the meta habitat extent, \
        or -- iff the location was local -- in the local habitat extent"
    )]
    fn sample_non_self_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &SpatiallyImplicitHabitat<M>,
        rng: &mut G,
    ) -> Location {
        // By PRE, location must be habitable, i.e. either in the local or the meta
        //  habitat
        if habitat.local().get_extent().contains(location) {
            if Bernoulli::sample_with(rng, self.local_migration_probability_per_generation.into()) {
                // Provide a dummpy Location in the meta community to disperse from
                // As the individual is dispersing to a different community,
                //  we can use standard dispersal in the meta community
                self.meta.sample_dispersal_from_location(
                    &Location::new(
                        habitat.meta().get_extent().x(),
                        habitat.meta().get_extent().y(),
                    ),
                    habitat.meta(),
                    rng,
                )
            } else {
                self.local
                    .sample_non_self_dispersal_from_location(location, habitat.local(), rng)
            }
        } else {
            self.meta
                .sample_non_self_dispersal_from_location(location, habitat.meta(), rng)
        }
    }

    #[must_use]
    fn get_self_dispersal_probability_at_location(
        &self,
        location: &Location,
        habitat: &SpatiallyImplicitHabitat<M>,
    ) -> ClosedUnitF64 {
        // By PRE, location must be habitable, i.e. either in the local or the meta
        //  habitat
        if habitat.local().get_extent().contains(location) {
            self.local
                .get_self_dispersal_probability_at_location(location, habitat.local())
                * ClosedUnitF64::from(self.local_migration_probability_per_generation).one_minus()
        } else {
            self.meta
                .get_self_dispersal_probability_at_location(location, habitat.meta())
        }
    }
}
