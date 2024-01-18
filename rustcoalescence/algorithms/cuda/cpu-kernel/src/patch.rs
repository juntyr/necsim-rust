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

use rust_cuda::{kernel::CompiledKernelPtx, lend::RustToCuda};

use rustcoalescence_algorithms_cuda_gpu_kernel::simulate;

use crate::SimulationKernelPtx;

// If `Kernel` is implemented for `ReportSpeciation` x `ReportDispersal`, i.e.
//  for {`False`, `True`} x {`False`, `True`} then it is implemented for all
//  `Boolean`s. However, Rust does not recognise that `Boolean` is closed over
//  {`False`, `True`}. This explicit impl provides the necessary coersion.

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
    >
    CompiledKernelPtx<
        simulate<M, H, G, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>,
    > for SimulationKernelPtx<M, H, G, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>
where
    crate::link::SimulationKernelPtx<M, H, G, S, X, D, C, T, N, E, I, A, False, False>:
        CompiledKernelPtx<simulate<M, H, G, S, X, D, C, T, N, E, I, A, False, False>>,
    crate::link::SimulationKernelPtx<M, H, G, S, X, D, C, T, N, E, I, A, False, True>:
        CompiledKernelPtx<simulate<M, H, G, S, X, D, C, T, N, E, I, A, False, True>>,
    crate::link::SimulationKernelPtx<M, H, G, S, X, D, C, T, N, E, I, A, True, False>:
        CompiledKernelPtx<simulate<M, H, G, S, X, D, C, T, N, E, I, A, True, False>>,
    crate::link::SimulationKernelPtx<M, H, G, S, X, D, C, T, N, E, I, A, True, True>:
        CompiledKernelPtx<simulate<M, H, G, S, X, D, C, T, N, E, I, A, True, True>>,
{
    #[inline]
    fn get_ptx() -> &'static CStr {
        match (ReportSpeciation::VALUE, ReportDispersal::VALUE) {
            (false, false) => crate::link::SimulationKernelPtx::<
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
                False,
                False,
            >::get_ptx(),
            (false, true) => crate::link::SimulationKernelPtx::<
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
                False,
                True,
            >::get_ptx(),
            (true, false) => crate::link::SimulationKernelPtx::<
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
                True,
                False,
            >::get_ptx(),
            (true, true) => crate::link::SimulationKernelPtx::<
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
                True,
                True,
            >::get_ptx(),
        }
    }

    #[inline]
    fn get_entry_point() -> &'static CStr {
        match (ReportSpeciation::VALUE, ReportDispersal::VALUE) {
            (false, false) => crate::link::SimulationKernelPtx::<
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
                False,
                False,
            >::get_entry_point(),
            (false, true) => crate::link::SimulationKernelPtx::<
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
                False,
                True,
            >::get_entry_point(),
            (true, false) => crate::link::SimulationKernelPtx::<
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
                True,
                False,
            >::get_entry_point(),
            (true, true) => crate::link::SimulationKernelPtx::<
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
                True,
                True,
            >::get_entry_point(),
        }
    }
}
