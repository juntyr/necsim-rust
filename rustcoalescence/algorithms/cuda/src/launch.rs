use std::marker::PhantomData;

use necsim_core::{
    lineage::GlobalLineageReference, reporter::Reporter, simulation::SimulationBuilder,
};
use necsim_core_bond::NonNegativeF64;

use necsim_impls_cuda::cogs::{maths::NvptxMathsCore, rng::CudaRng};
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
        rng::wyhash::WyHash,
    },
    parallelisation::Status,
};
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_algorithms::result::SimulationOutcome;
use rustcoalescence_scenarios::Scenario;

use rust_cuda::{
    common::RustToCuda,
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
    initialiser::CudaLineageStoreSampleInitialiser,
    kernel::SimulationKernel,
    parallelisation, CudaError,
};

#[allow(clippy::too_many_lines)]
pub fn initialise_and_simulate<
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
