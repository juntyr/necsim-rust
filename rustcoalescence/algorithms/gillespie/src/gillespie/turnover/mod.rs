use necsim_core::{
    cogs::{LocallyCoherentLineageStore, MathsCore},
    lineage::Lineage,
    reporter::Reporter,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_impls_no_std::cogs::{
    lineage_reference::in_memory::InMemoryLineageReference,
    lineage_store::coherent::locally::classical::ClassicalLineageStore,
    origin_sampler::pre_sampler::OriginPreSampler,
};
use necsim_impls_std::cogs::rng::pcg::Pcg;
use necsim_partitioning_core::{partition::Partition, LocalPartition};

use rustcoalescence_algorithms::{
    result::{ResumeError, SimulationOutcome},
    strategy::RestartFixUpStrategy,
    Algorithm,
};
use rustcoalescence_scenarios::Scenario;

use crate::arguments::get_effective_monolithic_partition;

use super::GillespieAlgorithm;

mod initialiser;
mod launch;

use initialiser::{
    fixup::FixUpInitialiser, genesis::GenesisInitialiser, resume::ResumeInitialiser,
};

// Default 'Gillespie' implementation for any turnover sampler
#[allow(clippy::type_complexity)]
impl<
        'p,
        O: Scenario<M, Pcg<M>, LineageReference = InMemoryLineageReference>,
        R: Reporter,
        P: LocalPartition<'p, R>,
        M: MathsCore,
    > Algorithm<'p, M, O, R, P> for GillespieAlgorithm
where
    O::LineageStore<ClassicalLineageStore<M, O::Habitat>>:
        LocallyCoherentLineageStore<M, O::Habitat, InMemoryLineageReference>,
{
    type LineageReference = InMemoryLineageReference;
    type LineageStore = O::LineageStore<ClassicalLineageStore<M, O::Habitat>>;
    type Rng = Pcg<M>;

    default fn get_effective_partition(args: &Self::Arguments, local_partition: &P) -> Partition {
        get_effective_monolithic_partition(args, local_partition)
    }

    #[allow(clippy::shadow_unrelated, clippy::too_many_lines)]
    default fn initialise_and_simulate<I: Iterator<Item = u64>>(
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
    default fn resume_and_simulate<
        I: Iterator<Item = u64>,
        L: ExactSizeIterator<Item = Lineage>,
    >(
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
    default fn fixup_for_restart<I: Iterator<Item = u64>, L: ExactSizeIterator<Item = Lineage>>(
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
