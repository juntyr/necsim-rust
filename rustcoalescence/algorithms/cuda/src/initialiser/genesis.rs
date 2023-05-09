use necsim_core::{
    cogs::{
        distribution::{Bernoulli, IndexUsize, UniformClosedOpenUnit},
        EmigrationExit, MathsCore, PrimeableRng, Rng, Samples,
    },
    lineage::Lineage,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::independent::{
        event_time_sampler::EventTimeSampler, IndependentActiveLineageSampler,
    },
    dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler,
    lineage_store::independent::IndependentLineageStore,
    origin_sampler::TrustedOriginSampler,
};

use rustcoalescence_scenarios::Scenario;

use rust_cuda::{common::RustToCuda, safety::NoAliasing};

use crate::CudaError;

use super::CudaLineageStoreSampleInitialiser;

#[allow(clippy::module_name_repetitions)]
pub struct GenesisInitialiser;

#[allow(clippy::trait_duplication_in_bounds)]
impl<
        M: MathsCore,
        G: Rng<M, Generator: PrimeableRng>
            + Samples<M, IndexUsize>
            + Samples<M, Bernoulli>
            + Samples<M, UniformClosedOpenUnit>
            + RustToCuda
            + NoAliasing,
        O: Scenario<M, G>,
    > CudaLineageStoreSampleInitialiser<M, G, O, CudaError> for GenesisInitialiser
where
    O::Habitat: RustToCuda + NoAliasing,
    O::DispersalSampler<InMemoryPackedAliasDispersalSampler<M, O::Habitat, G>>:
        RustToCuda + NoAliasing,
    O::TurnoverRate: RustToCuda + NoAliasing,
    O::SpeciationProbability: RustToCuda + NoAliasing,
{
    type ActiveLineageSampler<
        X: EmigrationExit<M, O::Habitat, G, IndependentLineageStore<M, O::Habitat>>
            + RustToCuda
            + NoAliasing,
        J: EventTimeSampler<M, O::Habitat, G, O::TurnoverRate> + RustToCuda + NoAliasing,
    > = IndependentActiveLineageSampler<
        M,
        O::Habitat,
        G,
        X,
        Self::DispersalSampler,
        O::TurnoverRate,
        O::SpeciationProbability,
        J,
    >;
    type DispersalSampler =
        O::DispersalSampler<InMemoryPackedAliasDispersalSampler<M, O::Habitat, G>>;

    fn init<
        'h,
        T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
        J: EventTimeSampler<M, O::Habitat, G, O::TurnoverRate> + RustToCuda + NoAliasing,
        X: EmigrationExit<M, O::Habitat, G, IndependentLineageStore<M, O::Habitat>>
            + RustToCuda
            + NoAliasing,
    >(
        self,
        origin_sampler: T,
        dispersal_sampler: O::DispersalSampler<
            InMemoryPackedAliasDispersalSampler<M, O::Habitat, G>,
        >,
        event_time_sampler: J,
    ) -> Result<
        (
            IndependentLineageStore<M, O::Habitat>,
            Self::DispersalSampler,
            Self::ActiveLineageSampler<X, J>,
            Vec<Lineage>,
            Vec<Lineage>,
        ),
        CudaError,
    >
    where
        O::Habitat: 'h,
    {
        let (lineage_store, active_lineage_sampler, lineages) =
            IndependentActiveLineageSampler::init_with_store_and_lineages(
                origin_sampler,
                event_time_sampler,
            );

        Ok((
            lineage_store,
            dispersal_sampler,
            active_lineage_sampler,
            lineages,
            Vec::new(),
        ))
    }
}
