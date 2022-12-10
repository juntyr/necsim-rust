use std::marker::PhantomData;

use necsim_core::{cogs::MathsCore, reporter::Reporter, simulation::SimulationBuilder};
use necsim_core_bond::NonNegativeF64;

use necsim_impls_cuda::cogs::rng::CudaRng;
use necsim_impls_no_std::{
    cogs::{
        active_lineage_sampler::independent::event_time_sampler::exp::ExpEventTimeSampler,
        coalescence_sampler::independent::IndependentCoalescenceSampler,
        dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler,
        emigration_exit::never::NeverEmigrationExit,
        event_sampler::independent::IndependentEventSampler,
        immigration_entry::never::NeverImmigrationEntry,
        lineage_store::independent::IndependentLineageStore,
        origin_sampler::{
            decomposition::DecompositionOriginSampler, pre_sampler::OriginPreSampler,
        },
        rng::{simple::SimpleRng, wyhash::WyHash},
    },
    parallelisation::Status,
};
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_algorithms::result::SimulationOutcome;
use rustcoalescence_scenarios::Scenario;

use rustcoalescence_algorithms_cuda_cpu_kernel::{
    BitonicGlobalSortStepKernel, BitonicSharedSortPrepKernel, BitonicSharedSortStepKernel,
    EvenOddSortKernel, SimulationKernel,
};
use rustcoalescence_algorithms_cuda_gpu_kernel::SimulatableKernel;

use rust_cuda::{
    common::RustToCuda,
    host::CudaDropWrapper,
    rustacuda::{
        function::{BlockSize, GridSize},
        prelude::{Stream, StreamFlags},
    },
};

use crate::{
    arguments::{
        CudaArguments, IsolatedParallelismMode, MonolithicParallelismMode, ParallelismMode,
    },
    cuda::with_initialised_cuda,
    error::CudaError,
    initialiser::CudaLineageStoreSampleInitialiser,
    parallelisation,
};

#[allow(clippy::too_many_lines, clippy::type_complexity)]
pub fn initialise_and_simulate<
    'p,
    M: MathsCore,
    O: Scenario<M, CudaRng<M, SimpleRng<M, WyHash>>>,
    R: Reporter,
    P: LocalPartition<'p, R>,
    I: Iterator<Item = u64>,
    L: CudaLineageStoreSampleInitialiser<M, CudaRng<M, SimpleRng<M, WyHash>>, O, Error>,
    Error: From<CudaError>,
>(
    args: &CudaArguments,
    rng: WyHash,
    scenario: O,
    pre_sampler: OriginPreSampler<I>,
    pause_before: Option<NonNegativeF64>,
    local_partition: &mut P,
    lineage_store_sampler_initialiser: L,
) -> Result<SimulationOutcome<WyHash>, Error>
where
    O::Habitat: RustToCuda,
    O::DispersalSampler<
        InMemoryPackedAliasDispersalSampler<M, O::Habitat, CudaRng<M, SimpleRng<M, WyHash>>>,
    >: RustToCuda,
    O::TurnoverRate: RustToCuda,
    O::SpeciationProbability: RustToCuda,
    SimulationKernel<
        M,
        O::Habitat,
        CudaRng<M, SimpleRng<M, WyHash>>,
        IndependentLineageStore<M, O::Habitat>,
        NeverEmigrationExit,
        L::DispersalSampler,
        IndependentCoalescenceSampler<M, O::Habitat>,
        O::TurnoverRate,
        O::SpeciationProbability,
        IndependentEventSampler<
            M,
            O::Habitat,
            CudaRng<M, SimpleRng<M, WyHash>>,
            NeverEmigrationExit,
            L::DispersalSampler,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        NeverImmigrationEntry,
        L::ActiveLineageSampler<NeverEmigrationExit, ExpEventTimeSampler>,
        R::ReportSpeciation,
        R::ReportDispersal,
    >: SimulatableKernel<
        M,
        O::Habitat,
        CudaRng<M, SimpleRng<M, WyHash>>,
        IndependentLineageStore<M, O::Habitat>,
        NeverEmigrationExit,
        L::DispersalSampler,
        IndependentCoalescenceSampler<M, O::Habitat>,
        O::TurnoverRate,
        O::SpeciationProbability,
        IndependentEventSampler<
            M,
            O::Habitat,
            CudaRng<M, SimpleRng<M, WyHash>>,
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
    let rng = CudaRng::from(SimpleRng::from(rng));

    let (
        habitat,
        dispersal_sampler,
        turnover_rate,
        speciation_probability,
        origin_sampler_auxiliary,
        decomposition_auxiliary,
    ) = scenario.build::<InMemoryPackedAliasDispersalSampler<
        M,
        O::Habitat,
        CudaRng<M, SimpleRng<M, WyHash>>,
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
        maths: PhantomData::<M>,
        habitat,
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
    let block_size = BlockSize::x(args.block_size.get());
    let grid_size = GridSize::x(args.grid_size.get());

    let event_slice = match args.parallelism_mode {
        ParallelismMode::Monolithic(MonolithicParallelismMode { event_slice })
        | ParallelismMode::IsolatedIndividuals(IsolatedParallelismMode { event_slice, .. })
        | ParallelismMode::IsolatedLandscape(IsolatedParallelismMode { event_slice, .. }) => {
            event_slice
        },
    };

    let (mut status, time, steps, lineages) = with_initialised_cuda(args.device, || {
        let stream = CudaDropWrapper::from(Stream::new(StreamFlags::NON_BLOCKING, None)?);

        let kernel = SimulationKernel::try_new(
            grid_size.clone(),
            block_size.clone(),
            args.ptx_jit,
            Box::new(|kernel| {
                crate::info::print_kernel_function_attributes("Simulation", kernel);
                Ok(())
            }),
        )?;

        let even_odd_sort_kernel = EvenOddSortKernel::try_new(
            GridSize::x(0),
            BlockSize::x(args.sort_block_size.get()),
            args.ptx_jit,
            Box::new(|kernel| {
                crate::info::print_kernel_function_attributes(
                    "Even Odd Sorting Global Step",
                    kernel,
                );
                Ok(())
            }),
        )?;

        let bitonic_sort_shared_prep_kernel = BitonicSharedSortPrepKernel::try_new(
            GridSize::x(0),
            args.ptx_jit,
            Box::new(|kernel| {
                crate::info::print_kernel_function_attributes(
                    "Bitonic Sorting Shared Prep",
                    kernel,
                );
                Ok(())
            }),
        )?;

        let bitonic_sort_shared_step_kernel = BitonicSharedSortStepKernel::try_new(
            GridSize::x(0),
            args.ptx_jit,
            Box::new(|kernel| {
                crate::info::print_kernel_function_attributes(
                    "Bitonic Sorting Shared Step",
                    kernel,
                );
                Ok(())
            }),
        )?;

        let bitonic_sort_global_step_kernel = BitonicGlobalSortStepKernel::try_new(
            GridSize::x(0),
            BlockSize::x(args.sort_block_size.get()),
            args.ptx_jit,
            Box::new(|kernel| {
                crate::info::print_kernel_function_attributes(
                    "Bitonic Sorting Global Step",
                    kernel,
                );
                Ok(())
            }),
        )?;

        parallelisation::monolithic::simulate(
            &mut simulation,
            kernel,
            (
                even_odd_sort_kernel,
                bitonic_sort_shared_prep_kernel,
                bitonic_sort_shared_step_kernel,
                bitonic_sort_global_step_kernel,
            ),
            (
                grid_size,
                block_size,
                args.dedup_cache,
                args.step_slice,
                args.sort_block_size.get() as usize,
                args.sort_mode,
                args.sort_batch_size.get(),
            ),
            &stream,
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
            rng: simulation.deconstruct().rng.into_inner().into_inner(),
        }),
    }
}
