#![deny(clippy::pedantic)]
#![feature(const_eval_limit)]
#![const_eval_limit = "1000000000000"]
#![feature(generic_associated_types)]
#![allow(incomplete_features)]
#![feature(specialization)]

#[macro_use]
extern crate serde_derive_state;

use necsim_core::{
    lineage::{GlobalLineageReference, Lineage},
    reporter::Reporter,
};
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
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_algorithms::{
    result::{ResumeError, SimulationOutcome},
    strategy::RestartFixUpStrategy,
    Algorithm, AlgorithmParamters,
};
use rustcoalescence_scenarios::Scenario;

use rust_cuda::common::RustToCuda;

mod arguments;
mod cuda;
mod info;
mod initialiser;
mod kernel;
mod launch;
mod parallelisation;

use crate::{
    arguments::CudaArguments,
    initialiser::{
        fixup::FixUpInitialiser, genesis::GenesisInitialiser, resume::ResumeInitialiser,
    },
    kernel::SimulationKernel,
};

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub struct CudaError(#[from] anyhow::Error);

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub enum CudaAlgorithm {}

impl AlgorithmParamters for CudaAlgorithm {
    type Arguments = CudaArguments;
    type Error = CudaError;
}

#[allow(clippy::type_complexity)]
impl<
        O: Scenario<NvptxMathsCore, CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>>,
        R: Reporter,
        P: LocalPartition<R>,
    > Algorithm<O, R, P> for CudaAlgorithm
where
    O::Habitat: RustToCuda,
    O::DispersalSampler<
        InMemoryPackedAliasDispersalSampler<
            NvptxMathsCore,
            O::Habitat,
            CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
        >,
    >: RustToCuda,
    O::TurnoverRate: RustToCuda,
    O::SpeciationProbability: RustToCuda,
    SimulationKernel<
        NvptxMathsCore,
        O::Habitat,
        CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
        GlobalLineageReference,
        IndependentLineageStore<NvptxMathsCore, O::Habitat>,
        NeverEmigrationExit,
        O::DispersalSampler<
            InMemoryPackedAliasDispersalSampler<
                NvptxMathsCore,
                O::Habitat,
                CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
            >,
        >,
        IndependentCoalescenceSampler<NvptxMathsCore, O::Habitat>,
        O::TurnoverRate,
        O::SpeciationProbability,
        IndependentEventSampler<
            NvptxMathsCore,
            O::Habitat,
            CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
            NeverEmigrationExit,
            O::DispersalSampler<
                InMemoryPackedAliasDispersalSampler<
                    NvptxMathsCore,
                    O::Habitat,
                    CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                >,
            >,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        NeverImmigrationEntry,
        IndependentActiveLineageSampler<
            NvptxMathsCore,
            O::Habitat,
            CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
            NeverEmigrationExit,
            O::DispersalSampler<
                InMemoryPackedAliasDispersalSampler<
                    NvptxMathsCore,
                    O::Habitat,
                    CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                >,
            >,
            O::TurnoverRate,
            O::SpeciationProbability,
            ExpEventTimeSampler,
        >,
        R::ReportSpeciation,
        R::ReportDispersal,
    >: rustcoalescence_algorithms_cuda_kernel::Kernel<
        NvptxMathsCore,
        O::Habitat,
        CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
        GlobalLineageReference,
        IndependentLineageStore<NvptxMathsCore, O::Habitat>,
        NeverEmigrationExit,
        O::DispersalSampler<
            InMemoryPackedAliasDispersalSampler<
                NvptxMathsCore,
                O::Habitat,
                CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
            >,
        >,
        IndependentCoalescenceSampler<NvptxMathsCore, O::Habitat>,
        O::TurnoverRate,
        O::SpeciationProbability,
        IndependentEventSampler<
            NvptxMathsCore,
            O::Habitat,
            CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
            NeverEmigrationExit,
            O::DispersalSampler<
                InMemoryPackedAliasDispersalSampler<
                    NvptxMathsCore,
                    O::Habitat,
                    CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                >,
            >,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        NeverImmigrationEntry,
        IndependentActiveLineageSampler<
            NvptxMathsCore,
            O::Habitat,
            CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
            NeverEmigrationExit,
            O::DispersalSampler<
                InMemoryPackedAliasDispersalSampler<
                    NvptxMathsCore,
                    O::Habitat,
                    CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                >,
            >,
            O::TurnoverRate,
            O::SpeciationProbability,
            ExpEventTimeSampler,
        >,
        R::ReportSpeciation,
        R::ReportDispersal,
    >,
    SimulationKernel<
        NvptxMathsCore,
        O::Habitat,
        CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
        GlobalLineageReference,
        IndependentLineageStore<NvptxMathsCore, O::Habitat>,
        NeverEmigrationExit,
        TrespassingDispersalSampler<
            NvptxMathsCore,
            O::Habitat,
            CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
            O::DispersalSampler<
                InMemoryPackedAliasDispersalSampler<
                    NvptxMathsCore,
                    O::Habitat,
                    CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                >,
            >,
            UniformAntiTrespassingDispersalSampler<
                NvptxMathsCore,
                O::Habitat,
                CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
            >,
        >,
        IndependentCoalescenceSampler<NvptxMathsCore, O::Habitat>,
        O::TurnoverRate,
        O::SpeciationProbability,
        IndependentEventSampler<
            NvptxMathsCore,
            O::Habitat,
            CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
            NeverEmigrationExit,
            TrespassingDispersalSampler<
                NvptxMathsCore,
                O::Habitat,
                CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                O::DispersalSampler<
                    InMemoryPackedAliasDispersalSampler<
                        NvptxMathsCore,
                        O::Habitat,
                        CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                    >,
                >,
                UniformAntiTrespassingDispersalSampler<
                    NvptxMathsCore,
                    O::Habitat,
                    CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                >,
            >,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        NeverImmigrationEntry,
        IndependentActiveLineageSampler<
            NvptxMathsCore,
            O::Habitat,
            CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
            NeverEmigrationExit,
            TrespassingDispersalSampler<
                NvptxMathsCore,
                O::Habitat,
                CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                O::DispersalSampler<
                    InMemoryPackedAliasDispersalSampler<
                        NvptxMathsCore,
                        O::Habitat,
                        CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                    >,
                >,
                UniformAntiTrespassingDispersalSampler<
                    NvptxMathsCore,
                    O::Habitat,
                    CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                >,
            >,
            O::TurnoverRate,
            O::SpeciationProbability,
            ConstEventTimeSampler,
        >,
        R::ReportSpeciation,
        R::ReportDispersal,
    >: rustcoalescence_algorithms_cuda_kernel::Kernel<
        NvptxMathsCore,
        O::Habitat,
        CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
        GlobalLineageReference,
        IndependentLineageStore<NvptxMathsCore, O::Habitat>,
        NeverEmigrationExit,
        TrespassingDispersalSampler<
            NvptxMathsCore,
            O::Habitat,
            CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
            O::DispersalSampler<
                InMemoryPackedAliasDispersalSampler<
                    NvptxMathsCore,
                    O::Habitat,
                    CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                >,
            >,
            UniformAntiTrespassingDispersalSampler<
                NvptxMathsCore,
                O::Habitat,
                CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
            >,
        >,
        IndependentCoalescenceSampler<NvptxMathsCore, O::Habitat>,
        O::TurnoverRate,
        O::SpeciationProbability,
        IndependentEventSampler<
            NvptxMathsCore,
            O::Habitat,
            CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
            NeverEmigrationExit,
            TrespassingDispersalSampler<
                NvptxMathsCore,
                O::Habitat,
                CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                O::DispersalSampler<
                    InMemoryPackedAliasDispersalSampler<
                        NvptxMathsCore,
                        O::Habitat,
                        CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                    >,
                >,
                UniformAntiTrespassingDispersalSampler<
                    NvptxMathsCore,
                    O::Habitat,
                    CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                >,
            >,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        NeverImmigrationEntry,
        IndependentActiveLineageSampler<
            NvptxMathsCore,
            O::Habitat,
            CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
            NeverEmigrationExit,
            TrespassingDispersalSampler<
                NvptxMathsCore,
                O::Habitat,
                CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                O::DispersalSampler<
                    InMemoryPackedAliasDispersalSampler<
                        NvptxMathsCore,
                        O::Habitat,
                        CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                    >,
                >,
                UniformAntiTrespassingDispersalSampler<
                    NvptxMathsCore,
                    O::Habitat,
                    CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
                >,
            >,
            O::TurnoverRate,
            O::SpeciationProbability,
            ConstEventTimeSampler,
        >,
        R::ReportSpeciation,
        R::ReportDispersal,
    >,
{
    type LineageReference = GlobalLineageReference;
    type LineageStore = IndependentLineageStore<Self::MathsCore, O::Habitat>;
    type MathsCore = NvptxMathsCore;
    type Rng = CudaRng<Self::MathsCore, WyHash<Self::MathsCore>>;

    fn initialise_and_simulate<I: Iterator<Item = u64>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<Self::MathsCore, I>,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<SimulationOutcome<Self::MathsCore, Self::Rng>, Self::Error> {
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
        pre_sampler: OriginPreSampler<Self::MathsCore, I>,
        lineages: L,
        resume_after: Option<NonNegativeF64>,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<SimulationOutcome<Self::MathsCore, Self::Rng>, ResumeError<Self::Error>> {
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
        pre_sampler: OriginPreSampler<Self::MathsCore, I>,
        lineages: L,
        restart_at: PositiveF64,
        fixup_strategy: RestartFixUpStrategy,
        local_partition: &mut P,
    ) -> Result<SimulationOutcome<Self::MathsCore, Self::Rng>, ResumeError<Self::Error>> {
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
