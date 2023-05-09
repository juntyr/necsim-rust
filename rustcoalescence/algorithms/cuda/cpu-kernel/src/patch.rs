use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageStore, MathsCore, PrimeableRng, Rng, SpeciationProbability, TurnoverRate,
    },
    reporter::boolean::{Boolean, False, True},
};
use necsim_impls_no_std::cogs::{
    active_lineage_sampler::singular::SingularActiveLineageSampler,
    event_sampler::tracking::MinSpeciationTrackingEventSampler,
};

use rust_cuda::{
    common::RustToCuda, host::TypedKernel, rustacuda::error::CudaResult, safety::NoAliasing,
};

use rustcoalescence_algorithms_cuda_gpu_kernel::{
    BitonicGlobalSortStepKernelPtx, BitonicGlobalSortSteppableKernel,
    BitonicSharedSortPrepKernelPtx, BitonicSharedSortPreparableKernel,
    BitonicSharedSortStepKernelPtx, BitonicSharedSortSteppableKernel, EvenOddSortKernelPtx,
    EvenOddSortableKernel, SimulatableKernel, SimulationKernelPtx,
};

use crate::{
    BitonicGlobalSortStepKernel, BitonicSharedSortPrepKernel, BitonicSharedSortStepKernel,
    EvenOddSortKernel, SimulationKernel,
};

// If `Kernel` is implemented for `ReportSpeciation` x `ReportDispersal`, i.e.
//  for {`False`, `True`} x {`False`, `True`} then it is implemented for all
//  `Boolean`s. However, Rust does not recognise that `Boolean` is closed over
//  {`False`, `True`}. These default impls provide the necessary coersion.

extern "C" {
    fn unreachable_cuda_simulation_linking_reporter() -> !;
}

#[allow(clippy::trait_duplication_in_bounds)]
unsafe impl<
        M: MathsCore,
        H: Habitat<M> + RustToCuda + NoAliasing,
        G: Rng<M, Generator: PrimeableRng> + RustToCuda + NoAliasing,
        S: LineageStore<M, H> + RustToCuda + NoAliasing,
        X: EmigrationExit<M, H, G, S> + RustToCuda + NoAliasing,
        D: DispersalSampler<M, H, G> + RustToCuda + NoAliasing,
        C: CoalescenceSampler<M, H, S> + RustToCuda + NoAliasing,
        T: TurnoverRate<M, H> + RustToCuda + NoAliasing,
        N: SpeciationProbability<M, H> + RustToCuda + NoAliasing,
        E: MinSpeciationTrackingEventSampler<M, H, G, S, X, D, C, T, N> + RustToCuda + NoAliasing,
        I: ImmigrationEntry<M> + RustToCuda + NoAliasing,
        A: SingularActiveLineageSampler<M, H, G, S, X, D, C, T, N, E, I> + RustToCuda + NoAliasing,
        ReportSpeciation: Boolean,
        ReportDispersal: Boolean,
    > SimulationKernelPtx<M, H, G, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>
    for SimulationKernel<M, H, G, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>
where
    SimulationKernel<M, H, G, S, X, D, C, T, N, E, I, A, False, False>:
        SimulationKernelPtx<M, H, G, S, X, D, C, T, N, E, I, A, False, False>,
    SimulationKernel<M, H, G, S, X, D, C, T, N, E, I, A, False, True>:
        SimulationKernelPtx<M, H, G, S, X, D, C, T, N, E, I, A, False, True>,
    SimulationKernel<M, H, G, S, X, D, C, T, N, E, I, A, True, False>:
        SimulationKernelPtx<M, H, G, S, X, D, C, T, N, E, I, A, True, False>,
    SimulationKernel<M, H, G, S, X, D, C, T, N, E, I, A, True, True>:
        SimulationKernelPtx<M, H, G, S, X, D, C, T, N, E, I, A, True, True>,
{
    default fn get_ptx_str() -> &'static str {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }

    default fn new_kernel() -> CudaResult<
        TypedKernel<
            dyn SimulatableKernel<
                M,
                H,
                G,
                S,
                X,
                D,
                C,
                T,
                N,
                E,
                I,
                A,
                ReportSpeciation,
                ReportDispersal,
            >,
        >,
    > {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }
}

#[allow(clippy::trait_duplication_in_bounds)]
unsafe impl<ReportSpeciation: Boolean, ReportDispersal: Boolean>
    EvenOddSortKernelPtx<ReportSpeciation, ReportDispersal>
    for EvenOddSortKernel<ReportSpeciation, ReportDispersal>
where
    EvenOddSortKernel<False, False>: EvenOddSortKernelPtx<False, False>,
    EvenOddSortKernel<False, True>: EvenOddSortKernelPtx<False, True>,
    EvenOddSortKernel<True, False>: EvenOddSortKernelPtx<True, False>,
    EvenOddSortKernel<True, True>: EvenOddSortKernelPtx<True, True>,
{
    default fn get_ptx_str() -> &'static str {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }

    default fn new_kernel(
    ) -> CudaResult<TypedKernel<dyn EvenOddSortableKernel<ReportSpeciation, ReportDispersal>>> {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }
}

#[allow(clippy::trait_duplication_in_bounds)]
unsafe impl<ReportSpeciation: Boolean, ReportDispersal: Boolean>
    BitonicGlobalSortStepKernelPtx<ReportSpeciation, ReportDispersal>
    for BitonicGlobalSortStepKernel<ReportSpeciation, ReportDispersal>
where
    BitonicGlobalSortStepKernel<False, False>: BitonicGlobalSortStepKernelPtx<False, False>,
    BitonicGlobalSortStepKernel<False, True>: BitonicGlobalSortStepKernelPtx<False, True>,
    BitonicGlobalSortStepKernel<True, False>: BitonicGlobalSortStepKernelPtx<True, False>,
    BitonicGlobalSortStepKernel<True, True>: BitonicGlobalSortStepKernelPtx<True, True>,
{
    default fn get_ptx_str() -> &'static str {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }

    default fn new_kernel() -> CudaResult<
        TypedKernel<dyn BitonicGlobalSortSteppableKernel<ReportSpeciation, ReportDispersal>>,
    > {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }
}

#[allow(clippy::trait_duplication_in_bounds)]
unsafe impl<ReportSpeciation: Boolean, ReportDispersal: Boolean>
    BitonicSharedSortStepKernelPtx<ReportSpeciation, ReportDispersal>
    for BitonicSharedSortStepKernel<ReportSpeciation, ReportDispersal>
where
    BitonicSharedSortStepKernel<False, False>: BitonicSharedSortStepKernelPtx<False, False>,
    BitonicSharedSortStepKernel<False, True>: BitonicSharedSortStepKernelPtx<False, True>,
    BitonicSharedSortStepKernel<True, False>: BitonicSharedSortStepKernelPtx<True, False>,
    BitonicSharedSortStepKernel<True, True>: BitonicSharedSortStepKernelPtx<True, True>,
{
    default fn get_ptx_str() -> &'static str {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }

    default fn new_kernel() -> CudaResult<
        TypedKernel<dyn BitonicSharedSortSteppableKernel<ReportSpeciation, ReportDispersal>>,
    > {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }
}

#[allow(clippy::trait_duplication_in_bounds)]
unsafe impl<ReportSpeciation: Boolean, ReportDispersal: Boolean>
    BitonicSharedSortPrepKernelPtx<ReportSpeciation, ReportDispersal>
    for BitonicSharedSortPrepKernel<ReportSpeciation, ReportDispersal>
where
    BitonicSharedSortPrepKernel<False, False>: BitonicSharedSortPrepKernelPtx<False, False>,
    BitonicSharedSortPrepKernel<False, True>: BitonicSharedSortPrepKernelPtx<False, True>,
    BitonicSharedSortPrepKernel<True, False>: BitonicSharedSortPrepKernelPtx<True, False>,
    BitonicSharedSortPrepKernel<True, True>: BitonicSharedSortPrepKernelPtx<True, True>,
{
    default fn get_ptx_str() -> &'static str {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }

    default fn new_kernel() -> CudaResult<
        TypedKernel<dyn BitonicSharedSortPreparableKernel<ReportSpeciation, ReportDispersal>>,
    > {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }
}
