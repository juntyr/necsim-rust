use necsim_core::{
    cogs::{Backup, DispersalSampler, Habitat, RngCore, SeparableDispersalSampler},
    landscape::Location,
};
use necsim_core_bond::{ClosedUnitF64, PositiveUnitF64};

use crate::cogs::{
    dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
    habitat::spatially_implicit::SpatiallyImplicitHabitat,
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
pub struct SpatiallyImplicitDispersalSampler<G: RngCore> {
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    local: NonSpatialDispersalSampler<G>,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    meta: NonSpatialDispersalSampler<G>,
    local_migration_probability_per_generation: PositiveUnitF64,
}

impl<G: RngCore> SpatiallyImplicitDispersalSampler<G> {
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
impl<G: RngCore> Backup for SpatiallyImplicitDispersalSampler<G> {
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
impl<G: RngCore> DispersalSampler<SpatiallyImplicitHabitat, G>
    for SpatiallyImplicitDispersalSampler<G>
{
    #[must_use]
    #[debug_requires(
        habitat.local().contains(location) || habitat.meta().contains(location),
        "location is inside either the local or meta habitat extent"
    )]
    #[debug_ensures(habitat.meta().contains(&ret) || if old(habitat.local().contains(location)) {
        habitat.local().contains(&ret)
    } else { false }, "target is inside the meta habitat extent, \
        or -- iff the location was local -- in the local habitat extent"
    )]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &SpatiallyImplicitHabitat,
        rng: &mut G,
    ) -> Location {
        use necsim_core::cogs::RngSampler;

        if habitat.local().contains(location) {
            if rng.sample_event(self.local_migration_probability_per_generation.into()) {
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
impl<G: RngCore> SeparableDispersalSampler<SpatiallyImplicitHabitat, G>
    for SpatiallyImplicitDispersalSampler<G>
{
    #[must_use]
    #[debug_requires(
        habitat.local().contains(location) || habitat.meta().contains(location),
        "location is inside either the local or meta habitat extent"
    )]
    #[debug_ensures(habitat.meta().contains(&ret) || if old(habitat.local().contains(location)) {
        habitat.local().contains(&ret)
    } else { false }, "target is inside the meta habitat extent, \
        or -- iff the location was local -- in the local habitat extent"
    )]
    fn sample_non_self_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &SpatiallyImplicitHabitat,
        rng: &mut G,
    ) -> Location {
        use necsim_core::cogs::RngSampler;

        if habitat.local().contains(location) {
            if rng.sample_event(self.local_migration_probability_per_generation.into()) {
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
    #[debug_requires(
        habitat.local().contains(location) || habitat.meta().contains(location),
        "location is inside either the local or meta habitat extent"
    )]
    fn get_self_dispersal_probability_at_location(
        &self,
        location: &Location,
        habitat: &SpatiallyImplicitHabitat,
    ) -> ClosedUnitF64 {
        if habitat.local().contains(location) {
            self.local
                .get_self_dispersal_probability_at_location(location, habitat.local())
                * ClosedUnitF64::from(self.local_migration_probability_per_generation).one_minus()
        } else {
            self.meta
                .get_self_dispersal_probability_at_location(location, habitat.meta())
        }
    }
}
