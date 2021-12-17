#![deny(clippy::pedantic)]
#![feature(const_eval_limit)]
#![const_eval_limit = "1000000000000"]
#![allow(incomplete_features)]
#![feature(specialization)]

#[macro_use]
extern crate serde_derive_state;

use std::marker::PhantomData;

use necsim_core::{
    lineage::{GlobalLineageReference, Lineage},
    reporter::Reporter,
    simulation::SimulationBuilder,
};
use necsim_core_bond::NonNegativeF64;

use necsim_impls_cuda::cogs::{maths::NvptxMathsCore, rng::CudaRng};
use necsim_impls_no_std::{
    cogs::{
        active_lineage_sampler::independent::{
            event_time_sampler::exp::ExpEventTimeSampler, IndependentActiveLineageSampler,
        },
        coalescence_sampler::independent::IndependentCoalescenceSampler,
        dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler,
        emigration_exit::never::NeverEmigrationExit,
        event_sampler::independent::IndependentEventSampler,
        immigration_entry::never::NeverImmigrationEntry,
        lineage_store::independent::IndependentLineageStore,
        origin_sampler::{
            decomposition::DecompositionOriginSampler, pre_sampler::OriginPreSampler,
        },
        rng::wyhash::WyHash,
    },
    parallelisation::Status,
};
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_algorithms::{Algorithm, AlgorithmParamters, AlgorithmResult};
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

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub enum CudaAlgorithm {}

impl AlgorithmParamters for CudaAlgorithm {
    type Arguments = CudaArguments;
    type Error = anyhow::Error;
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
    ) -> Result<AlgorithmResult<Self::MathsCore, Self::Rng>, Self::Error> {
        let lineages: Vec<Lineage> = match args.parallelism_mode {
            // Apply no lineage origin partitioning in the `Monolithic` mode
            ParallelismMode::Monolithic(..) => scenario.sample_habitat(pre_sampler).collect(),
            // Apply lineage origin partitioning in the `IsolatedIndividuals` mode
            ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode { partition, .. }) => {
                scenario
                    .sample_habitat(pre_sampler.partition(partition))
                    .collect()
            },
            // Apply lineage origin partitioning in the `IsolatedLandscape` mode
            ParallelismMode::IsolatedLandscape(IsolatedParallelismMode { partition, .. }) => {
                DecompositionOriginSampler::new(
                    scenario.sample_habitat(pre_sampler),
                    &O::decompose(scenario.habitat(), partition),
                )
                .collect()
            },
        };

        let (habitat, dispersal_sampler, turnover_rate, speciation_probability) =
            scenario.build::<InMemoryPackedAliasDispersalSampler<
                Self::MathsCore,
                O::Habitat,
                CudaRng<Self::MathsCore, WyHash<Self::MathsCore>>,
            >>();
        let lineage_store = IndependentLineageStore::default();
        let emigration_exit = NeverEmigrationExit::default();
        let coalescence_sampler = IndependentCoalescenceSampler::default();
        let event_sampler = IndependentEventSampler::default();
        let immigration_entry = NeverImmigrationEntry::default();

        let active_lineage_sampler =
            IndependentActiveLineageSampler::empty(ExpEventTimeSampler::new(args.delta_t));

        let mut simulation = SimulationBuilder {
            maths: PhantomData::<Self::MathsCore>,
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
            | ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode {
                event_slice, ..
            })
            | ParallelismMode::IsolatedLandscape(IsolatedParallelismMode { event_slice, .. }) => {
                event_slice
            },
        };

        let (status, time, steps, lineages) = with_initialised_cuda(args.device, || {
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
        })?;

        match status {
            Status::Done => Ok(AlgorithmResult::Done { time, steps }),
            Status::Paused => Ok(AlgorithmResult::Paused {
                time,
                steps,
                lineages: lineages.into_iter().collect(),
                rng: simulation.rng_mut().clone(),
                marker: PhantomData,
            }),
        }
    }
}
