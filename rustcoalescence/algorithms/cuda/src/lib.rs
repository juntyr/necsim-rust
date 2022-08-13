#![deny(clippy::pedantic)]
#![feature(generic_associated_types)]
#![allow(incomplete_features)]
#![feature(inline_const_pat)]

#[macro_use]
extern crate serde_derive_state;

use necsim_core::{cogs::MathsCore, lineage::Lineage, reporter::Reporter};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_impls_cuda::cogs::{maths::NvptxMathsCore, rng::CudaRng};
use necsim_impls_no_std::cogs::{
    active_lineage_sampler::independent::{
        event_time_sampler::{exp::ExpEventTimeSampler, r#const::ConstEventTimeSampler},
        IndependentActiveLineageSampler,
    },
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    dispersal_sampler::{
        in_memory::packed_alias::InMemoryPackedAliasDispersalSampler,
        trespassing::{
            uniform::UniformAntiTrespassingDispersalSampler, TrespassingDispersalSampler,
        },
    },
    emigration_exit::never::NeverEmigrationExit,
    event_sampler::independent::IndependentEventSampler,
    immigration_entry::never::NeverImmigrationEntry,
    lineage_store::independent::IndependentLineageStore,
    origin_sampler::pre_sampler::OriginPreSampler,
    rng::wyhash::WyHash,
};
use necsim_partitioning_core::{partition::Partition, LocalPartition};

use rustcoalescence_algorithms::{
    result::{ResumeError, SimulationOutcome},
    strategy::RestartFixUpStrategy,
    Algorithm, AlgorithmDefaults, AlgorithmParamters,
};
use rustcoalescence_scenarios::Scenario;

use rustcoalescence_algorithms_cuda_cpu_kernel::SimulationKernel;
use rustcoalescence_algorithms_cuda_gpu_kernel::SimulatableKernel;

use rust_cuda::common::RustToCuda;

mod arguments;
mod cuda;
mod error;
mod info;
mod initialiser;
mod launch;
mod parallelisation;

use crate::{
    arguments::{CudaArguments, IsolatedParallelismMode, ParallelismMode},
    error::CudaError,
    initialiser::{
        fixup::FixUpInitialiser, genesis::GenesisInitialiser, resume::ResumeInitialiser,
    },
};

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub enum CudaAlgorithm {}

impl AlgorithmParamters for CudaAlgorithm {
    type Arguments = CudaArguments;
    type Error = CudaError;
}

impl AlgorithmDefaults for CudaAlgorithm {
    type MathsCore = NvptxMathsCore;
}

#[allow(clippy::trait_duplication_in_bounds)]
impl<
        'p,
        M: MathsCore,
        O: Scenario<M, CudaRng<M, WyHash<M>>>,
        R: Reporter,
        P: LocalPartition<'p, R>,
    > Algorithm<'p, M, O, R, P> for CudaAlgorithm
where
    O::Habitat: RustToCuda,
    O::DispersalSampler<InMemoryPackedAliasDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>>:
        RustToCuda,
    O::TurnoverRate: RustToCuda,
    O::SpeciationProbability: RustToCuda,
    SimulationKernel<
        M,
        O::Habitat,
        CudaRng<M, WyHash<M>>,
        IndependentLineageStore<M, O::Habitat>,
        NeverEmigrationExit,
        O::DispersalSampler<
            InMemoryPackedAliasDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
        >,
        IndependentCoalescenceSampler<M, O::Habitat>,
        O::TurnoverRate,
        O::SpeciationProbability,
        IndependentEventSampler<
            M,
            O::Habitat,
            CudaRng<M, WyHash<M>>,
            NeverEmigrationExit,
            O::DispersalSampler<
                InMemoryPackedAliasDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
            >,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        NeverImmigrationEntry,
        IndependentActiveLineageSampler<
            M,
            O::Habitat,
            CudaRng<M, WyHash<M>>,
            NeverEmigrationExit,
            O::DispersalSampler<
                InMemoryPackedAliasDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
            >,
            O::TurnoverRate,
            O::SpeciationProbability,
            ExpEventTimeSampler,
        >,
        R::ReportSpeciation,
        R::ReportDispersal,
    >: SimulatableKernel<
        M,
        O::Habitat,
        CudaRng<M, WyHash<M>>,
        IndependentLineageStore<M, O::Habitat>,
        NeverEmigrationExit,
        O::DispersalSampler<
            InMemoryPackedAliasDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
        >,
        IndependentCoalescenceSampler<M, O::Habitat>,
        O::TurnoverRate,
        O::SpeciationProbability,
        IndependentEventSampler<
            M,
            O::Habitat,
            CudaRng<M, WyHash<M>>,
            NeverEmigrationExit,
            O::DispersalSampler<
                InMemoryPackedAliasDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
            >,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        NeverImmigrationEntry,
        IndependentActiveLineageSampler<
            M,
            O::Habitat,
            CudaRng<M, WyHash<M>>,
            NeverEmigrationExit,
            O::DispersalSampler<
                InMemoryPackedAliasDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
            >,
            O::TurnoverRate,
            O::SpeciationProbability,
            ExpEventTimeSampler,
        >,
        R::ReportSpeciation,
        R::ReportDispersal,
    >,
    SimulationKernel<
        M,
        O::Habitat,
        CudaRng<M, WyHash<M>>,
        IndependentLineageStore<M, O::Habitat>,
        NeverEmigrationExit,
        TrespassingDispersalSampler<
            M,
            O::Habitat,
            CudaRng<M, WyHash<M>>,
            O::DispersalSampler<
                InMemoryPackedAliasDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
            >,
            UniformAntiTrespassingDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
        >,
        IndependentCoalescenceSampler<M, O::Habitat>,
        O::TurnoverRate,
        O::SpeciationProbability,
        IndependentEventSampler<
            M,
            O::Habitat,
            CudaRng<M, WyHash<M>>,
            NeverEmigrationExit,
            TrespassingDispersalSampler<
                M,
                O::Habitat,
                CudaRng<M, WyHash<M>>,
                O::DispersalSampler<
                    InMemoryPackedAliasDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
                >,
                UniformAntiTrespassingDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
            >,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        NeverImmigrationEntry,
        IndependentActiveLineageSampler<
            M,
            O::Habitat,
            CudaRng<M, WyHash<M>>,
            NeverEmigrationExit,
            TrespassingDispersalSampler<
                M,
                O::Habitat,
                CudaRng<M, WyHash<M>>,
                O::DispersalSampler<
                    InMemoryPackedAliasDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
                >,
                UniformAntiTrespassingDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
            >,
            O::TurnoverRate,
            O::SpeciationProbability,
            ConstEventTimeSampler,
        >,
        R::ReportSpeciation,
        R::ReportDispersal,
    >: SimulatableKernel<
        M,
        O::Habitat,
        CudaRng<M, WyHash<M>>,
        IndependentLineageStore<M, O::Habitat>,
        NeverEmigrationExit,
        TrespassingDispersalSampler<
            M,
            O::Habitat,
            CudaRng<M, WyHash<M>>,
            O::DispersalSampler<
                InMemoryPackedAliasDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
            >,
            UniformAntiTrespassingDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
        >,
        IndependentCoalescenceSampler<M, O::Habitat>,
        O::TurnoverRate,
        O::SpeciationProbability,
        IndependentEventSampler<
            M,
            O::Habitat,
            CudaRng<M, WyHash<M>>,
            NeverEmigrationExit,
            TrespassingDispersalSampler<
                M,
                O::Habitat,
                CudaRng<M, WyHash<M>>,
                O::DispersalSampler<
                    InMemoryPackedAliasDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
                >,
                UniformAntiTrespassingDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
            >,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        NeverImmigrationEntry,
        IndependentActiveLineageSampler<
            M,
            O::Habitat,
            CudaRng<M, WyHash<M>>,
            NeverEmigrationExit,
            TrespassingDispersalSampler<
                M,
                O::Habitat,
                CudaRng<M, WyHash<M>>,
                O::DispersalSampler<
                    InMemoryPackedAliasDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
                >,
                UniformAntiTrespassingDispersalSampler<M, O::Habitat, CudaRng<M, WyHash<M>>>,
            >,
            O::TurnoverRate,
            O::SpeciationProbability,
            ConstEventTimeSampler,
        >,
        R::ReportSpeciation,
        R::ReportDispersal,
    >,
{
    type LineageStore = IndependentLineageStore<M, O::Habitat>;
    type Rng = CudaRng<M, WyHash<M>>;

    fn get_logical_partition(args: &Self::Arguments, _local_partition: &P) -> Partition {
        match &args.parallelism_mode {
            ParallelismMode::Monolithic(_) => Partition::monolithic(),
            ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode { partition, .. })
            | ParallelismMode::IsolatedLandscape(IsolatedParallelismMode { partition, .. }) => {
                *partition
            },
        }
    }

    fn initialise_and_simulate<I: Iterator<Item = u64>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<M, I>,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<SimulationOutcome<M, Self::Rng>, Self::Error> {
        launch::initialise_and_simulate(
            &args,
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
            &args,
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
            &args,
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
