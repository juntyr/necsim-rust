use necsim_core::{
    cogs::{
        rng::{Event, IndexUsize, UniformClosedOpenUnit},
        DistributionSampler, EmigrationExit, MathsCore, PrimeableRng, Rng,
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

use rust_cuda::common::RustToCuda;

use crate::CudaError;

use super::CudaLineageStoreSampleInitialiser;

#[allow(clippy::module_name_repetitions)]
pub struct GenesisInitialiser;

impl<M: MathsCore, G: Rng<M, Generator: PrimeableRng> + RustToCuda, O: Scenario<M, G>>
    CudaLineageStoreSampleInitialiser<M, G, O, CudaError> for GenesisInitialiser
where
    O::Habitat: RustToCuda,
    O::DispersalSampler<InMemoryPackedAliasDispersalSampler<M, O::Habitat, G>>: RustToCuda,
    O::TurnoverRate: RustToCuda,
    O::SpeciationProbability: RustToCuda,
    G::Sampler: DistributionSampler<M, G::Generator, G::Sampler, IndexUsize>
        + DistributionSampler<M, G::Generator, G::Sampler, Event>
        + DistributionSampler<M, G::Generator, G::Sampler, UniformClosedOpenUnit>,
{
    type ActiveLineageSampler<
        X: EmigrationExit<M, O::Habitat, G, IndependentLineageStore<M, O::Habitat>> + RustToCuda,
        J: EventTimeSampler<M, O::Habitat, G, O::TurnoverRate> + RustToCuda,
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
        J: EventTimeSampler<M, O::Habitat, G, O::TurnoverRate> + RustToCuda,
        X: EmigrationExit<M, O::Habitat, G, IndependentLineageStore<M, O::Habitat>> + RustToCuda,
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
