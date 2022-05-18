#![deny(clippy::pedantic)]
#![feature(const_eval_limit)]
#![const_eval_limit = "1000000000000"]
#![feature(associated_type_bounds)]
#![allow(incomplete_features)]
#![feature(specialization)]

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageStore, MathsCore, PrimeableRng, Rng, SpeciationProbability, TurnoverRate,
    },
    reporter::boolean::Boolean,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::singular::SingularActiveLineageSampler,
    event_sampler::tracking::MinSpeciationTrackingEventSampler,
};

use rust_cuda::{
    common::RustToCuda,
    host::{CudaDropWrapper, LaunchConfig, LaunchPackage, Launcher, TypedKernel},
    rustacuda::{
        error::CudaResult,
        function::{BlockSize, Function, GridSize},
        stream::Stream,
    },
};

use rustcoalescence_algorithms_cuda_gpu_kernel::SimulatableKernel;

mod link;
mod patch;

pub type KernelCompilationCallback = dyn FnMut(&Function) -> CudaResult<()>;

#[allow(clippy::module_name_repetitions)]
pub struct SimulationKernel<
    M: MathsCore,
    H: Habitat<M> + RustToCuda,
    G: Rng<M, Generator: PrimeableRng> + RustToCuda,
    S: LineageStore<M, H> + RustToCuda,
    X: EmigrationExit<M, H, G, S> + RustToCuda,
    D: DispersalSampler<M, H, G> + RustToCuda,
    C: CoalescenceSampler<M, H, S> + RustToCuda,
    T: TurnoverRate<M, H> + RustToCuda,
    N: SpeciationProbability<M, H> + RustToCuda,
    E: MinSpeciationTrackingEventSampler<M, H, G, S, X, D, C, T, N> + RustToCuda,
    I: ImmigrationEntry<M> + RustToCuda,
    A: SingularActiveLineageSampler<M, H, G, S, X, D, C, T, N, E, I> + RustToCuda,
    ReportSpeciation: Boolean,
    ReportDispersal: Boolean,
> {
    #[allow(clippy::type_complexity)]
    kernel: TypedKernel<
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
    stream: CudaDropWrapper<Stream>,
    grid: GridSize,
    block: BlockSize,
    ptx_jit: bool,
    watcher: Box<KernelCompilationCallback>,
}

impl<
        M: MathsCore,
        H: Habitat<M> + RustToCuda,
        G: Rng<M, Generator: PrimeableRng> + RustToCuda,
        S: LineageStore<M, H> + RustToCuda,
        X: EmigrationExit<M, H, G, S> + RustToCuda,
        D: DispersalSampler<M, H, G> + RustToCuda,
        C: CoalescenceSampler<M, H, S> + RustToCuda,
        T: TurnoverRate<M, H> + RustToCuda,
        N: SpeciationProbability<M, H> + RustToCuda,
        E: MinSpeciationTrackingEventSampler<M, H, G, S, X, D, C, T, N> + RustToCuda,
        I: ImmigrationEntry<M> + RustToCuda,
        A: SingularActiveLineageSampler<M, H, G, S, X, D, C, T, N, E, I> + RustToCuda,
        ReportSpeciation: Boolean,
        ReportDispersal: Boolean,
    > SimulationKernel<M, H, G, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>
{
    /// # Errors
    ///
    /// Returns a `CudaError` if loading the CUDA kernel failed.
    pub fn try_new(
        stream: Stream,
        grid: GridSize,
        block: BlockSize,
        ptx_jit: bool,
        on_compile: Box<KernelCompilationCallback>,
    ) -> CudaResult<Self>
    where
        Self: SimulatableKernel<
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
    {
        let stream = CudaDropWrapper::from(stream);
        let kernel = Self::new_kernel()?;

        Ok(Self {
            kernel,
            stream,
            grid,
            block,
            ptx_jit,
            watcher: on_compile,
        })
    }
}

impl<
        M: MathsCore,
        H: Habitat<M> + RustToCuda,
        G: Rng<M, Generator: PrimeableRng> + RustToCuda,
        S: LineageStore<M, H> + RustToCuda,
        X: EmigrationExit<M, H, G, S> + RustToCuda,
        D: DispersalSampler<M, H, G> + RustToCuda,
        C: CoalescenceSampler<M, H, S> + RustToCuda,
        T: TurnoverRate<M, H> + RustToCuda,
        N: SpeciationProbability<M, H> + RustToCuda,
        E: MinSpeciationTrackingEventSampler<M, H, G, S, X, D, C, T, N> + RustToCuda,
        I: ImmigrationEntry<M> + RustToCuda,
        A: SingularActiveLineageSampler<M, H, G, S, X, D, C, T, N, E, I> + RustToCuda,
        ReportSpeciation: Boolean,
        ReportDispersal: Boolean,
    > Launcher
    for SimulationKernel<M, H, G, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>
{
    type CompilationWatcher = Box<KernelCompilationCallback>;
    type KernelTraitObject = dyn SimulatableKernel<
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
    >;

    fn get_launch_package(&mut self) -> LaunchPackage<Self> {
        LaunchPackage {
            config: LaunchConfig {
                grid: self.grid.clone(),
                block: self.block.clone(),
                shared_memory_size: 0_u32,
                ptx_jit: self.ptx_jit,
            },

            kernel: &mut self.kernel,
            stream: &mut self.stream,

            watcher: &mut self.watcher,
        }
    }

    fn on_compile(kernel: &Function, watcher: &mut Self::CompilationWatcher) -> CudaResult<()> {
        (watcher)(kernel)
    }
}
