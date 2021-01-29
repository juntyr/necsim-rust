use std::marker::PhantomData;

use necsim_core::cogs::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
    LineageReference, LineageStore, MinSpeciationTrackingEventSampler, PrimeableRng,
    SingularActiveLineageSampler, SpeciationProbability,
};

use rustacuda::{function::Function, module::Module};
use rustacuda_core::DeviceCopy;

use rust_cuda::common::RustToCuda;

use ptx_jit::host::compiler::PtxJITCompiler;

mod r#impl;
mod launch;
mod specialiser;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::type_complexity)]
pub struct SimulationKernel<
    'k,
    H: Habitat + RustToCuda,
    G: PrimeableRng<H> + RustToCuda,
    N: SpeciationProbability<H> + RustToCuda,
    D: DispersalSampler<H, G> + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    S: LineageStore<H, R> + RustToCuda,
    X: EmigrationExit<H, G, N, D, R, S> + RustToCuda,
    C: CoalescenceSampler<H, R, S> + RustToCuda,
    E: MinSpeciationTrackingEventSampler<H, G, N, D, R, S, X, C> + RustToCuda,
    I: ImmigrationEntry + RustToCuda,
    A: SingularActiveLineageSampler<H, G, N, D, R, S, X, C, E, I> + RustToCuda,
    const REPORT_SPECIATION: bool,
    const REPORT_DISPERSAL: bool,
> {
    compiler: &'k mut PtxJITCompiler,
    module: &'k mut Module,
    entry_point: &'k mut Function<'k>,
    marker: PhantomData<(H, G, N, D, R, S, X, C, E, I, A)>,
}
