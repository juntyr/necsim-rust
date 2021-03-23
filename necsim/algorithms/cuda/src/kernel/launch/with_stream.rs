use std::marker::PhantomData;

use necsim_core::cogs::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
    LineageReference, LineageStore, MinSpeciationTrackingEventSampler, PrimeableRng,
    SingularActiveLineageSampler, SpeciationProbability, TurnoverRate,
};

use rustacuda::{
    function::{BlockSize, Function, GridSize},
    module::Module,
    stream::Stream,
};
use rustacuda_core::DeviceCopy;

use ptx_jit::host::compiler::PtxJITCompiler;
use rust_cuda::common::RustToCuda;

use super::SimulationKernelWithDimensions;

#[allow(clippy::type_complexity)]
pub struct SimulationKernelWithDimensionsStream<
    'k,
    's,
    H: Habitat + RustToCuda,
    G: PrimeableRng<H> + RustToCuda,
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
    const REPORT_SPECIATION: bool,
    const REPORT_DISPERSAL: bool,
> {
    pub(super) compiler: &'k mut PtxJITCompiler,
    pub(super) ptx_jit: bool,
    pub(super) module: &'k mut Module,
    pub(super) entry_point: &'k mut Function<'k>,
    pub(super) marker: PhantomData<(H, G, R, S, X, D, C, T, N, E, I, A)>,
    pub(super) grid_size: GridSize,
    pub(super) block_size: BlockSize,
    pub(super) shared_mem_bytes: u32,
    pub(super) stream: &'s Stream,
}

impl<
        'k,
        H: Habitat + RustToCuda,
        G: PrimeableRng<H> + RustToCuda,
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
        const REPORT_SPECIATION: bool,
        const REPORT_DISPERSAL: bool,
    >
    SimulationKernelWithDimensions<
        'k,
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
        REPORT_SPECIATION,
        REPORT_DISPERSAL,
    >
{
    #[allow(clippy::type_complexity)]
    pub fn with_stream<'s>(
        self,
        stream: &'s Stream,
    ) -> SimulationKernelWithDimensionsStream<
        'k,
        's,
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
        REPORT_SPECIATION,
        REPORT_DISPERSAL,
    > {
        SimulationKernelWithDimensionsStream {
            compiler: self.compiler,
            ptx_jit: self.ptx_jit,
            module: self.module,
            entry_point: self.entry_point,
            marker: self.marker,
            grid_size: self.grid_size.clone(),
            block_size: self.block_size.clone(),
            shared_mem_bytes: self.shared_mem_bytes,
            stream,
        }
    }
}
