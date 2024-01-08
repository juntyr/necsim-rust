use std::ffi::CStr;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageStore, MathsCore, PrimeableRng, SpeciationProbability, TurnoverRate,
    },
    reporter::boolean::{Boolean, False, True},
};
use necsim_impls_no_std::cogs::{
    active_lineage_sampler::singular::SingularActiveLineageSampler,
    event_sampler::tracking::MinSpeciationTrackingEventSampler,
};

use rust_cuda::{lend::RustToCuda, kernel::CompiledKernelPtx};

use rustcoalescence_algorithms_cuda_gpu_kernel::simulate;

use crate::SimulationKernelPtx;

// If `Kernel` is implemented for `ReportSpeciation` x `ReportDispersal`, i.e.
//  for {`False`, `True`} x {`False`, `True`} then it is implemented for all
//  `Boolean`s. However, Rust does not recognise that `Boolean` is closed over
//  {`False`, `True`}. These default impls provide the necessary coersion.

extern "C" {
    fn unreachable_cuda_simulation_linking_reporter() -> !;
}

#[allow(clippy::trait_duplication_in_bounds)]
unsafe impl<
        M: MathsCore + Sync,
        H: Habitat<M> + RustToCuda + Sync,
        G: PrimeableRng<M> + RustToCuda + Sync,
        S: LineageStore<M, H> + RustToCuda + Sync,
        X: EmigrationExit<M, H, G, S> + RustToCuda + Sync,
        D: DispersalSampler<M, H, G> + RustToCuda + Sync,
        C: CoalescenceSampler<M, H, S> + RustToCuda + Sync,
        T: TurnoverRate<M, H> + RustToCuda + Sync,
        N: SpeciationProbability<M, H> + RustToCuda + Sync,
        E: MinSpeciationTrackingEventSampler<M, H, G, S, X, D, C, T, N> + RustToCuda + Sync,
        I: ImmigrationEntry<M> + RustToCuda + Sync,
        A: SingularActiveLineageSampler<M, H, G, S, X, D, C, T, N, E, I> + RustToCuda + Sync,
        ReportSpeciation: Boolean,
        ReportDispersal: Boolean,
    > CompiledKernelPtx<simulate<M, H, G, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>>
    for SimulationKernelPtx<M, H, G, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>
where
    SimulationKernelPtx<M, H, G, S, X, D, C, T, N, E, I, A, False, False>:
        CompiledKernelPtx<simulate<M, H, G, S, X, D, C, T, N, E, I, A, False, False>>,
    SimulationKernelPtx<M, H, G, S, X, D, C, T, N, E, I, A, False, True>:
        CompiledKernelPtx<simulate<M, H, G, S, X, D, C, T, N, E, I, A, False, True>>,
    SimulationKernelPtx<M, H, G, S, X, D, C, T, N, E, I, A, True, False>:
        CompiledKernelPtx<simulate<M, H, G, S, X, D, C, T, N, E, I, A, True, False>>,
    SimulationKernelPtx<M, H, G, S, X, D, C, T, N, E, I, A, True, True>:
        CompiledKernelPtx<simulate<M, H, G, S, X, D, C, T, N, E, I, A, True, True>>,
{
    default fn get_ptx() -> &'static CStr {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }

    default fn get_entry_point() -> &'static CStr {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }
}
