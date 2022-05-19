use necsim_core::{
    cogs::{
        distribution::{Bernoulli, IndexU64},
        Backup, DispersalSampler, DistributionSampler, Habitat, MathsCore, Rng,
        SampledDistribution, SeparableDispersalSampler,
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
pub struct SpatiallyImplicitDispersalSampler<M: MathsCore, G: Rng<M>>
where
    G::Sampler: DistributionSampler<M, G::Generator, G::Sampler, IndexU64>
        + DistributionSampler<M, G::Generator, G::Sampler, Bernoulli>,
{
    #[cfg_attr(feature = "cuda", cuda(embed))]
    local: NonSpatialDispersalSampler<M, G>,
    #[cfg_attr(feature = "cuda", cuda(embed))]
    meta: NonSpatialDispersalSampler<M, G>,
    local_migration_probability_per_generation: PositiveUnitF64,
}

impl<M: MathsCore, G: Rng<M>> SpatiallyImplicitDispersalSampler<M, G>
where
    G::Sampler: DistributionSampler<M, G::Generator, G::Sampler, IndexU64>
        + DistributionSampler<M, G::Generator, G::Sampler, Bernoulli>,
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

#[contract_trait]
impl<M: MathsCore, G: Rng<M>> Backup for SpatiallyImplicitDispersalSampler<M, G>
where
    G::Sampler: DistributionSampler<M, G::Generator, G::Sampler, IndexU64>
        + DistributionSampler<M, G::Generator, G::Sampler, Bernoulli>,
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

#[contract_trait]
impl<M: MathsCore, G: Rng<M>> DispersalSampler<M, SpatiallyImplicitHabitat<M>, G>
    for SpatiallyImplicitDispersalSampler<M, G>
where
    G::Sampler: DistributionSampler<M, G::Generator, G::Sampler, IndexU64>
        + DistributionSampler<M, G::Generator, G::Sampler, Bernoulli>,
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

#[contract_trait]
impl<M: MathsCore, G: Rng<M>> SeparableDispersalSampler<M, SpatiallyImplicitHabitat<M>, G>
    for SpatiallyImplicitDispersalSampler<M, G>
where
    G::Sampler: DistributionSampler<M, G::Generator, G::Sampler, IndexU64>
        + DistributionSampler<M, G::Generator, G::Sampler, Bernoulli>,
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
