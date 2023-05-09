use necsim_core::{
    cogs::{
        distribution::{Bernoulli, IndexUsize, UniformClosedOpenUnit},
        EmigrationExit, MathsCore, PrimeableRng, Rng, Samples,
    },
    lineage::Lineage,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::{
        independent::{
            event_time_sampler::{r#const::ConstEventTimeSampler, EventTimeSampler},
            IndependentActiveLineageSampler,
        },
        resuming::lineage::{ExceptionalLineage, SplitExceptionalLineages},
    },
    dispersal_sampler::{
        in_memory::packed_alias::InMemoryPackedAliasDispersalSampler,
        trespassing::{
            uniform::UniformAntiTrespassingDispersalSampler, TrespassingDispersalSampler,
        },
    },
    lineage_store::independent::IndependentLineageStore,
    origin_sampler::{resuming::ResumingOriginSampler, TrustedOriginSampler},
};

use rustcoalescence_algorithms::{
    result::ResumeError,
    strategy::{OutOfDemeStrategy, OutOfHabitatStrategy, RestartFixUpStrategy},
};
use rustcoalescence_scenarios::Scenario;

use rust_cuda::{common::RustToCuda, safety::NoAliasing};

use crate::CudaError;

use super::CudaLineageStoreSampleInitialiser;

pub struct FixUpInitialiser<L: ExactSizeIterator<Item = Lineage>> {
    pub lineages: L,
    pub restart_at: PositiveF64,
    pub fixup_strategy: RestartFixUpStrategy,
}

#[allow(clippy::trait_duplication_in_bounds)]
impl<
        L: ExactSizeIterator<Item = Lineage>,
        M: MathsCore,
        G: Rng<M, Generator: PrimeableRng>
            + Samples<M, IndexUsize>
            + Samples<M, Bernoulli>
            + Samples<M, UniformClosedOpenUnit>
            + RustToCuda
            + NoAliasing,
        O: Scenario<M, G>,
    > CudaLineageStoreSampleInitialiser<M, G, O, ResumeError<CudaError>> for FixUpInitialiser<L>
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
        ConstEventTimeSampler,
    >;
    type DispersalSampler = TrespassingDispersalSampler<
        M,
        O::Habitat,
        G,
        O::DispersalSampler<InMemoryPackedAliasDispersalSampler<M, O::Habitat, G>>,
        UniformAntiTrespassingDispersalSampler<M, O::Habitat, G>,
    >;

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
        _event_time_sampler: J,
    ) -> Result<
        (
            IndependentLineageStore<M, O::Habitat>,
            Self::DispersalSampler,
            Self::ActiveLineageSampler<X, J>,
            Vec<Lineage>,
            Vec<Lineage>,
        ),
        ResumeError<CudaError>,
    >
    where
        O::Habitat: 'h,
    {
        let habitat = origin_sampler.habitat();
        let pre_sampler = origin_sampler.into_pre_sampler();

        let (lineage_store, active_lineage_sampler, mut good_lineages, exceptional_lineages) =
            IndependentActiveLineageSampler::resume_with_store_and_lineages(
                ResumingOriginSampler::new(habitat, pre_sampler, self.lineages),
                ConstEventTimeSampler::new(self.restart_at),
                NonNegativeF64::zero(),
            );

        let SplitExceptionalLineages {
            coalescence,
            out_of_deme,
            out_of_habitat,
        } = ExceptionalLineage::split_vec(exceptional_lineages);

        let mut exceptional_lineages = Vec::new();
        let mut fixable_lineages = Vec::new();

        // The Independent algorithm can deal with late coalescence
        good_lineages.extend(coalescence.into_iter().map(|(lineage, _)| lineage));

        match self.fixup_strategy.out_of_deme {
            OutOfDemeStrategy::Abort => {
                exceptional_lineages
                    .extend(out_of_deme.into_iter().map(ExceptionalLineage::OutOfDeme));
            },
            OutOfDemeStrategy::Dispersal => {
                fixable_lineages.extend(out_of_deme.into_iter());
            },
        }

        match self.fixup_strategy.out_of_habitat {
            OutOfHabitatStrategy::Abort => {
                exceptional_lineages.extend(
                    out_of_habitat
                        .into_iter()
                        .map(ExceptionalLineage::OutOfHabitat),
                );
            },
            OutOfHabitatStrategy::UniformDispersal => {
                fixable_lineages.extend(out_of_habitat.into_iter());
            },
        }

        if !exceptional_lineages.is_empty() {
            return Err(ResumeError::Sample(exceptional_lineages));
        }

        let dispersal_sampler = TrespassingDispersalSampler::new(
            dispersal_sampler,
            UniformAntiTrespassingDispersalSampler::default(),
        );

        // Simulate the fixable lineages, pass through the good ones
        Ok((
            lineage_store,
            dispersal_sampler,
            active_lineage_sampler,
            fixable_lineages,
            good_lineages,
        ))
    }
}
