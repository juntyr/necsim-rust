use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageReference, LineageStore, MinSpeciationTrackingEventSampler,
        PeekableActiveLineageSampler, PrimeableRng, SingularActiveLineageSampler,
        SpeciationProbability, TurnoverRate,
    },
    reporter::boolean::Boolean,
};

use rust_cuda::{
    common::RustToCuda,
    host::{CudaDropWrapper, LaunchConfig, Launcher, TypedKernel},
    rustacuda::{
        error::CudaResult,
        function::{BlockSize, Function, GridSize},
        stream::Stream,
    },
    rustacuda_core::DeviceCopy,
};

use rustcoalescence_algorithms_cuda_kernel::Kernel;

mod link;

#[allow(clippy::type_complexity, clippy::module_name_repetitions)]
pub struct SimulationKernel<
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
    A: SingularActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
        + PeekableActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
        + RustToCuda,
    ReportSpeciation: Boolean,
    ReportDispersal: Boolean,
> {
    kernel: TypedKernel<
        dyn Kernel<H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>,
    >,
    stream: CudaDropWrapper<Stream>,
    grid: GridSize,
    block: BlockSize,
}

impl<
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
        A: SingularActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
            + PeekableActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
            + RustToCuda,
        ReportSpeciation: Boolean,
        ReportDispersal: Boolean,
    > SimulationKernel<H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>
{
    pub fn try_new(stream: Stream, grid: GridSize, block: BlockSize) -> CudaResult<Self>
    where
        Self: Kernel<H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>,
    {
        let stream = CudaDropWrapper::from(stream);
        let kernel = Self::new_kernel()?;

        Ok(Self {
            kernel,
            stream,
            grid,
            block,
        })
    }
}

impl<
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
        A: SingularActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
            + PeekableActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
            + RustToCuda,
        ReportSpeciation: Boolean,
        ReportDispersal: Boolean,
    > Launcher
    for SimulationKernel<H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>
{
    type KernelTraitObject =
        dyn Kernel<H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>;

    fn get_config(&self) -> LaunchConfig {
        LaunchConfig {
            grid: self.grid.clone(),
            block: self.block.clone(),
            shared_memory_size: 0_u32,
        }
    }

    fn get_stream(&self) -> &Stream {
        &self.stream
    }

    fn get_kernel_mut(&mut self) -> &mut TypedKernel<Self::KernelTraitObject> {
        &mut self.kernel
    }

    fn on_compile(&mut self, kernel: &Function) -> CudaResult<()> {
        crate::info::print_kernel_function_attributes(kernel);

        Ok(())
    }
}
