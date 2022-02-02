#![deny(clippy::pedantic)]
#![feature(const_eval_limit)]
#![const_eval_limit = "1000000000000"]
#![feature(generic_associated_types)]
#![allow(incomplete_features)]
#![feature(specialization)]

#[macro_use]
extern crate serde_derive_state;

use std::marker::PhantomData;

use necsim_core::{
    cogs::{DispersalSampler, EmigrationExit, MathsCore, PrimeableRng},
    lineage::{GlobalLineageReference, Lineage},
    reporter::Reporter,
    simulation::SimulationBuilder,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_impls_cuda::cogs::{maths::NvptxMathsCore, rng::CudaRng};
use necsim_impls_no_std::{
    cogs::{
        active_lineage_sampler::{
            independent::{
                event_time_sampler::{
                    exp::ExpEventTimeSampler, r#const::ConstEventTimeSampler, EventTimeSampler,
                },
                IndependentActiveLineageSampler,
            },
            resuming::lineage::{ExceptionalLineage, SplitExceptionalLineages},
            singular::SingularActiveLineageSampler,
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
        origin_sampler::{
            decomposition::DecompositionOriginSampler, pre_sampler::OriginPreSampler,
            resuming::ResumingOriginSampler, TrustedOriginSampler,
        },
        rng::wyhash::WyHash,
    },
    parallelisation::Status,
};
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_algorithms::{
    result::{ResumeError, SimulationOutcome},
    strategy::{OutOfDemeStrategy, OutOfHabitatStrategy, RestartFixUpStrategy},
    Algorithm, AlgorithmParamters,
};
use rustcoalescence_scenarios::Scenario;

use rust_cuda::{
    common::RustToCuda,
    rustacuda::{
        function::{BlockSize, GridSize},
        prelude::{Stream, StreamFlags},
    },
};

mod arguments;
mod cuda;
mod info;
mod kernel;
mod parallelisation;

use arguments::{
    CudaArguments, IsolatedParallelismMode, MonolithicParallelismMode, ParallelismMode,
};

use cuda::with_initialised_cuda;

use crate::kernel::SimulationKernel;

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
        struct GenesisInitialiser;

        impl<M: MathsCore, G: PrimeableRng<M> + RustToCuda, O: Scenario<M, G>>
            CudaLineageStoreSampleInitialiser<M, G, O, CudaError> for GenesisInitialiser
        where
            O::Habitat: RustToCuda,
            O::DispersalSampler<InMemoryPackedAliasDispersalSampler<M, O::Habitat, G>>: RustToCuda,
            O::TurnoverRate: RustToCuda,
            O::SpeciationProbability: RustToCuda,
        {
            type ActiveLineageSampler<
                X: EmigrationExit<
                        M,
                        O::Habitat,
                        G,
                        GlobalLineageReference,
                        IndependentLineageStore<M, O::Habitat>,
                    > + RustToCuda,
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
                X: EmigrationExit<
                        M,
                        O::Habitat,
                        G,
                        GlobalLineageReference,
                        IndependentLineageStore<M, O::Habitat>,
                    > + RustToCuda,
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

        initialise_and_simulate(
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
        struct ResumeInitialiser<L: ExactSizeIterator<Item = Lineage>> {
            lineages: L,
            resume_after: Option<NonNegativeF64>,
        }

        impl<
                L: ExactSizeIterator<Item = Lineage>,
                M: MathsCore,
                G: PrimeableRng<M> + RustToCuda,
                O: Scenario<M, G>,
            > CudaLineageStoreSampleInitialiser<M, G, O, ResumeError<CudaError>>
            for ResumeInitialiser<L>
        where
            O::Habitat: RustToCuda,
            O::DispersalSampler<InMemoryPackedAliasDispersalSampler<M, O::Habitat, G>>: RustToCuda,
            O::TurnoverRate: RustToCuda,
            O::SpeciationProbability: RustToCuda,
        {
            type ActiveLineageSampler<
                X: EmigrationExit<
                        M,
                        O::Habitat,
                        G,
                        GlobalLineageReference,
                        IndependentLineageStore<M, O::Habitat>,
                    > + RustToCuda,
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
                X: EmigrationExit<
                        M,
                        O::Habitat,
                        G,
                        GlobalLineageReference,
                        IndependentLineageStore<M, O::Habitat>,
                    > + RustToCuda,
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
                ResumeError<CudaError>,
            >
            where
                O::Habitat: 'h,
            {
                let habitat = origin_sampler.habitat();
                let pre_sampler = origin_sampler.into_pre_sampler();

                let (lineage_store, active_lineage_sampler, mut lineages, mut exceptional_lineages) =
                    IndependentActiveLineageSampler::resume_with_store_and_lineages(
                        ResumingOriginSampler::new(habitat, pre_sampler, self.lineages),
                        event_time_sampler,
                        self.resume_after.unwrap_or(NonNegativeF64::zero()),
                    );

                // The Independent algorithm can deal with late coalescence
                lineages.extend(ExceptionalLineage::drain_coalescing_lineages(
                    &mut exceptional_lineages,
                ));

                if !exceptional_lineages.is_empty() {
                    return Err(ResumeError::Sample(exceptional_lineages));
                }

                Ok((
                    lineage_store,
                    dispersal_sampler,
                    active_lineage_sampler,
                    lineages,
                    Vec::new(),
                ))
            }
        }

        initialise_and_simulate(
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
        struct FixUpInitialiser<L: ExactSizeIterator<Item = Lineage>> {
            lineages: L,
            restart_at: PositiveF64,
            fixup_strategy: RestartFixUpStrategy,
        }

        impl<
                L: ExactSizeIterator<Item = Lineage>,
                M: MathsCore,
                G: PrimeableRng<M> + RustToCuda,
                O: Scenario<M, G>,
            > CudaLineageStoreSampleInitialiser<M, G, O, ResumeError<CudaError>>
            for FixUpInitialiser<L>
        where
            O::Habitat: RustToCuda,
            O::DispersalSampler<InMemoryPackedAliasDispersalSampler<M, O::Habitat, G>>: RustToCuda,
            O::TurnoverRate: RustToCuda,
            O::SpeciationProbability: RustToCuda,
        {
            type ActiveLineageSampler<
                X: EmigrationExit<
                        M,
                        O::Habitat,
                        G,
                        GlobalLineageReference,
                        IndependentLineageStore<M, O::Habitat>,
                    > + RustToCuda,
                J: EventTimeSampler<M, O::Habitat, G, O::TurnoverRate> + RustToCuda,
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
                J: EventTimeSampler<M, O::Habitat, G, O::TurnoverRate> + RustToCuda,
                X: EmigrationExit<
                        M,
                        O::Habitat,
                        G,
                        GlobalLineageReference,
                        IndependentLineageStore<M, O::Habitat>,
                    > + RustToCuda,
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

                let (
                    lineage_store,
                    active_lineage_sampler,
                    mut good_lineages,
                    exceptional_lineages,
                ) = IndependentActiveLineageSampler::resume_with_store_and_lineages(
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

        initialise_and_simulate(
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

#[allow(clippy::too_many_lines)]
fn initialise_and_simulate<
    O: Scenario<NvptxMathsCore, CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>>,
    R: Reporter,
    P: LocalPartition<R>,
    I: Iterator<Item = u64>,
    L: CudaLineageStoreSampleInitialiser<
        NvptxMathsCore,
        CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
        O,
        Error,
    >,
    Error: From<CudaError>,
>(
    args: &CudaArguments,
    rng: CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
    scenario: O,
    pre_sampler: OriginPreSampler<NvptxMathsCore, I>,
    pause_before: Option<NonNegativeF64>,
    local_partition: &mut P,
    lineage_store_sampler_initialiser: L,
) -> Result<SimulationOutcome<NvptxMathsCore, CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>>, Error>
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
        L::DispersalSampler,
        IndependentCoalescenceSampler<NvptxMathsCore, O::Habitat>,
        O::TurnoverRate,
        O::SpeciationProbability,
        IndependentEventSampler<
            NvptxMathsCore,
            O::Habitat,
            CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
            NeverEmigrationExit,
            L::DispersalSampler,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        NeverImmigrationEntry,
        L::ActiveLineageSampler<NeverEmigrationExit, ExpEventTimeSampler>,
        R::ReportSpeciation,
        R::ReportDispersal,
    >: rustcoalescence_algorithms_cuda_kernel::Kernel<
        NvptxMathsCore,
        O::Habitat,
        CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
        GlobalLineageReference,
        IndependentLineageStore<NvptxMathsCore, O::Habitat>,
        NeverEmigrationExit,
        L::DispersalSampler,
        IndependentCoalescenceSampler<NvptxMathsCore, O::Habitat>,
        O::TurnoverRate,
        O::SpeciationProbability,
        IndependentEventSampler<
            NvptxMathsCore,
            O::Habitat,
            CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
            NeverEmigrationExit,
            L::DispersalSampler,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        NeverImmigrationEntry,
        L::ActiveLineageSampler<NeverEmigrationExit, ExpEventTimeSampler>,
        R::ReportSpeciation,
        R::ReportDispersal,
    >,
{
    let (
        habitat,
        dispersal_sampler,
        turnover_rate,
        speciation_probability,
        origin_sampler_auxiliary,
        decomposition_auxiliary,
    ) = scenario.build::<InMemoryPackedAliasDispersalSampler<
        NvptxMathsCore,
        O::Habitat,
        CudaRng<NvptxMathsCore, WyHash<NvptxMathsCore>>,
    >>();
    let coalescence_sampler = IndependentCoalescenceSampler::default();
    let event_sampler = IndependentEventSampler::default();

    let (lineage_store, dispersal_sampler, active_lineage_sampler, lineages, passthrough) =
        match args.parallelism_mode {
            // Apply no lineage origin partitioning in the `Monolithic` mode
            ParallelismMode::Monolithic(..) => lineage_store_sampler_initialiser.init(
                O::sample_habitat(&habitat, pre_sampler, origin_sampler_auxiliary),
                dispersal_sampler,
                ExpEventTimeSampler::new(args.delta_t),
            )?,
            // Apply lineage origin partitioning in the `IsolatedIndividuals` mode
            ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode { partition, .. }) => {
                lineage_store_sampler_initialiser.init(
                    O::sample_habitat(
                        &habitat,
                        pre_sampler.partition(partition),
                        origin_sampler_auxiliary,
                    ),
                    dispersal_sampler,
                    ExpEventTimeSampler::new(args.delta_t),
                )?
            },
            // Apply lineage origin partitioning in the `IsolatedLandscape` mode
            ParallelismMode::IsolatedLandscape(IsolatedParallelismMode { partition, .. }) => {
                lineage_store_sampler_initialiser.init(
                    DecompositionOriginSampler::new(
                        O::sample_habitat(&habitat, pre_sampler, origin_sampler_auxiliary),
                        &O::decompose(&habitat, partition, decomposition_auxiliary),
                    ),
                    dispersal_sampler,
                    ExpEventTimeSampler::new(args.delta_t),
                )?
            },
        };

    let emigration_exit = NeverEmigrationExit::default();
    let immigration_entry = NeverImmigrationEntry::default();

    let mut simulation = SimulationBuilder {
        maths: PhantomData::<NvptxMathsCore>,
        habitat,
        lineage_reference: PhantomData::<GlobalLineageReference>,
        lineage_store,
        dispersal_sampler,
        coalescence_sampler,
        turnover_rate,
        speciation_probability,
        emigration_exit,
        event_sampler,
        active_lineage_sampler,
        rng,
        immigration_entry,
    }
    .build();

    // Note: It seems to be more performant to spawn smaller blocks
    let block_size = BlockSize::x(args.block_size);
    let grid_size = GridSize::x(args.grid_size);

    let event_slice = match args.parallelism_mode {
        ParallelismMode::Monolithic(MonolithicParallelismMode { event_slice })
        | ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode { event_slice, .. })
        | ParallelismMode::IsolatedLandscape(IsolatedParallelismMode { event_slice, .. }) => {
            event_slice
        },
    };

    let (mut status, time, steps, lineages) = with_initialised_cuda(args.device, || {
        let kernel = SimulationKernel::try_new(
            Stream::new(StreamFlags::NON_BLOCKING, None)?,
            grid_size.clone(),
            block_size.clone(),
        )?;

        parallelisation::monolithic::simulate(
            &mut simulation,
            kernel,
            (grid_size, block_size, args.dedup_cache, args.step_slice),
            lineages,
            event_slice,
            pause_before,
            local_partition,
        )
    })
    .map_err(CudaError::from)?;

    if !passthrough.is_empty() {
        status = Status::Paused;
    }

    match status {
        Status::Done => Ok(SimulationOutcome::Done { time, steps }),
        Status::Paused => Ok(SimulationOutcome::Paused {
            time,
            steps,
            lineages: lineages
                .into_iter()
                .chain(passthrough.into_iter())
                .collect(),
            rng: simulation.rng_mut().clone(),
            marker: PhantomData,
        }),
    }
}

#[allow(clippy::type_complexity)]
trait CudaLineageStoreSampleInitialiser<
    M: MathsCore,
    G: PrimeableRng<M> + RustToCuda,
    O: Scenario<M, G>,
    Error: From<CudaError>,
> where
    O::Habitat: RustToCuda,
    O::DispersalSampler<InMemoryPackedAliasDispersalSampler<M, O::Habitat, G>>: RustToCuda,
    O::TurnoverRate: RustToCuda,
    O::SpeciationProbability: RustToCuda,
{
    type DispersalSampler: DispersalSampler<M, O::Habitat, G> + RustToCuda;
    type ActiveLineageSampler<
        X: EmigrationExit<
            M,
            O::Habitat,
            G,
            GlobalLineageReference,
            IndependentLineageStore<M, O::Habitat>,
        > + RustToCuda,
        J: EventTimeSampler<M, O::Habitat, G, O::TurnoverRate> + RustToCuda,
    >: SingularActiveLineageSampler<
        M, O::Habitat, G, GlobalLineageReference, IndependentLineageStore<M, O::Habitat>,
        X, Self::DispersalSampler, IndependentCoalescenceSampler<M, O::Habitat>, O::TurnoverRate,
        O::SpeciationProbability, IndependentEventSampler<
            M, O::Habitat, G, X, Self::DispersalSampler, O::TurnoverRate, O::SpeciationProbability
        >, NeverImmigrationEntry,
    > + RustToCuda;

    fn init<
        'h,
        T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
        J: EventTimeSampler<M, O::Habitat, G, O::TurnoverRate> + RustToCuda,
        X: EmigrationExit<
                M,
                O::Habitat,
                G,
                GlobalLineageReference,
                IndependentLineageStore<M, O::Habitat>,
            > + RustToCuda,
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
        Error,
    >
    where
        O::Habitat: 'h;
}
