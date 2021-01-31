use std::marker::PhantomData;

use necsim_core::cogs::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
    LineageReference, LineageStore, MinSpeciationTrackingEventSampler, PrimeableRng,
    SingularActiveLineageSampler, SpeciationProbability,
};

use rustacuda::{
    function::{BlockSize, Function, GridSize},
    module::Module,
};
use rustacuda_core::DeviceCopy;

use ptx_jit::host::compiler::PtxJITCompiler;
use rust_cuda::common::RustToCuda;

use super::SimulationKernel;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::type_complexity)]
pub struct SimulationKernelWithDimensions<
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
    pub(super) compiler: &'k mut PtxJITCompiler,
    pub(super) ptx_jit: bool,
    pub(super) module: &'k mut Module,
    pub(super) entry_point: &'k mut Function<'k>,
    pub(super) marker: PhantomData<(H, G, N, D, R, S, X, C, E, I, A)>,
    pub(super) grid_size: GridSize,
    pub(super) block_size: BlockSize,
    pub(super) shared_mem_bytes: u32,
}

impl<
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
    > SimulationKernel<'k, H, G, N, D, R, S, X, C, E, I, A, REPORT_SPECIATION, REPORT_DISPERSAL>
{
    #[allow(clippy::type_complexity)]
    pub fn with_dimensions<GS: Into<GridSize>, BS: Into<BlockSize>>(
        &'k mut self,
        grid_size: GS,
        block_size: BS,
        shared_mem_bytes: u32,
    ) -> SimulationKernelWithDimensions<
        'k,
        H,
        G,
        N,
        D,
        R,
        S,
        X,
        C,
        E,
        I,
        A,
        REPORT_SPECIATION,
        REPORT_DISPERSAL,
    > {
        SimulationKernelWithDimensions {
            compiler: self.compiler,
            ptx_jit: self.ptx_jit,
            module: self.module,
            entry_point: self.entry_point,
            marker: self.marker,
            grid_size: grid_size.into(),
            block_size: block_size.into(),
            shared_mem_bytes,
        }
    }
}
