use necsim_core::{
    cogs::{GloballyCoherentLineageStore, MathsCore, SeparableDispersalSampler},
    lineage::Lineage,
    reporter::Reporter,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_impls_no_std::cogs::{
    dispersal_sampler::in_memory::separable_alias::InMemorySeparableAliasDispersalSampler,
    lineage_reference::in_memory::InMemoryLineageReference,
    lineage_store::coherent::globally::gillespie::GillespieLineageStore,
    maths::intrinsics::IntrinsicsMathsCore, origin_sampler::pre_sampler::OriginPreSampler,
};
use necsim_impls_std::cogs::rng::pcg::Pcg;
use necsim_partitioning_core::{partition::Partition, LocalPartition};

use rustcoalescence_algorithms::{
    result::{ResumeError, SimulationOutcome},
    strategy::RestartFixUpStrategy,
    Algorithm, AlgorithmDefaults, AlgorithmParamters,
};
use rustcoalescence_scenarios::Scenario;

use crate::arguments::{get_gillespie_logical_partition, GillespieArguments};

mod initialiser;
mod launch;

use initialiser::{
    fixup::FixUpInitialiser, genesis::GenesisInitialiser, resume::ResumeInitialiser,
};

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub struct EventSkippingAlgorithm {}

impl AlgorithmParamters for EventSkippingAlgorithm {
    type Arguments = GillespieArguments;
    type Error = !;
}

impl AlgorithmDefaults for EventSkippingAlgorithm {
    type MathsCore = IntrinsicsMathsCore;
}

#[allow(clippy::type_complexity)]
impl<
        'p,
        O: Scenario<M, Pcg<M>, LineageReference = InMemoryLineageReference>,
        R: Reporter,
        P: LocalPartition<'p, R>,
        M: MathsCore,
    > Algorithm<'p, M, O, R, P> for EventSkippingAlgorithm
where
    O::LineageStore<GillespieLineageStore<M, O::Habitat>>:
        GloballyCoherentLineageStore<M, O::Habitat, InMemoryLineageReference>,
    O::DispersalSampler<InMemorySeparableAliasDispersalSampler<M, O::Habitat, Pcg<M>>>:
        SeparableDispersalSampler<M, O::Habitat, Pcg<M>>,
{
    type LineageReference = InMemoryLineageReference;
    type LineageStore = O::LineageStore<GillespieLineageStore<M, O::Habitat>>;
    type Rng = Pcg<M>;

    fn get_logical_partition(args: &Self::Arguments, local_partition: &P) -> Partition {
        get_gillespie_logical_partition(args, local_partition)
    }

    #[allow(clippy::shadow_unrelated, clippy::too_many_lines)]
    fn initialise_and_simulate<I: Iterator<Item = u64>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<M, I>,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<SimulationOutcome<M, Self::Rng>, Self::Error> {
        launch::initialise_and_simulate(
            args,
            rng,
            scenario,
            pre_sampler,
            pause_before,
            local_partition,
            GenesisInitialiser,
        )
    }

    /// # Errors
    ///
    /// Returns a `ContinueError::Sample` if initialising the resuming
    ///  simulation failed
    #[allow(clippy::too_many_lines)]
    fn resume_and_simulate<I: Iterator<Item = u64>, L: ExactSizeIterator<Item = Lineage>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<M, I>,
        lineages: L,
        resume_after: Option<NonNegativeF64>,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<SimulationOutcome<M, Self::Rng>, ResumeError<Self::Error>> {
        launch::initialise_and_simulate(
            args,
            rng,
            scenario,
            pre_sampler,
            pause_before,
            local_partition,
            ResumeInitialiser {
                lineages,
                resume_after,
            },
        )
    }

    /// # Errors
    ///
    /// Returns a `ContinueError<Self::Error>` if fixing up the restarting
    ///  simulation (incl. running the algorithm) failed
    #[allow(clippy::too_many_lines)]
    fn fixup_for_restart<I: Iterator<Item = u64>, L: ExactSizeIterator<Item = Lineage>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<M, I>,
        lineages: L,
        restart_at: PositiveF64,
        fixup_strategy: RestartFixUpStrategy,
        local_partition: &mut P,
    ) -> Result<SimulationOutcome<M, Self::Rng>, ResumeError<Self::Error>> {
        launch::initialise_and_simulate(
            args,
            rng,
            scenario,
            pre_sampler,
            Some(PositiveF64::max_after(restart_at.into(), restart_at.into()).into()),
            local_partition,
            FixUpInitialiser {
                lineages,
                restart_at,
                fixup_strategy,
            },
        )
    }
}
