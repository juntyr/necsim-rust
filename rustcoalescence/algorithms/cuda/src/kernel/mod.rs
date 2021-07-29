use std::marker::PhantomData;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageReference, LineageStore, MinSpeciationTrackingEventSampler, PrimeableRng,
        SingularActiveLineageSampler, SpeciationProbability, TurnoverRate,
    },
    reporter::boolean::Boolean,
};

use rustcoalescence_algorithms_cuda_kernel_ptx_jit::host::compiler::PtxJITCompiler;

use rust_cuda::{
    rustacuda::{function::Function, module::Module},
    rustacuda_core::DeviceCopy,
};

use rust_cuda::common::RustToCuda;

mod r#impl;
mod launch;
mod specialiser;

pub mod dummy;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::type_complexity)]
pub struct SimulationKernel<
    'k,
    H: Habitat + RustToCuda,
    G: PrimeableRng + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    S: LineageStore<H, R> + RustToCuda,
    X: EmigrationExit<H, G, R, S> + RustToCuda,
    D: DispersalSampler<H, G> + RustToCuda,
    C: CoalescenceSampler<H, R, S> + RustToCuda,
    T: TurnoverRate<H> + RustToCuda,
    N: SpeciationProbability<H> + RustToCuda,
    E: MinSpeciationTrackingEventSampler<H, G, R, S, X, D, C, T, N> + RustToCuda,
    I: ImmigrationEntry + RustToCuda,
    A: SingularActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I> + RustToCuda,
    ReportSpeciation: Boolean,
    ReportDispersal: Boolean,
> {
    compiler: &'k mut PtxJITCompiler,
    ptx_jit: bool,
    module: &'k mut Module,
    entry_point: &'k mut Function<'k>,
    marker: PhantomData<(
        H,
        G,
        R,
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
    )>,
}
