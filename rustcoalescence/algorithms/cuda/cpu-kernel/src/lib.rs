#![deny(clippy::pedantic)]
#![feature(const_eval_limit)]
#![const_eval_limit = "1000000000000"]
#![feature(associated_type_bounds)]
#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(generic_const_exprs)]

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageStore, MathsCore, PrimeableRng, Rng, SpeciationProbability, TurnoverRate,
    },
    reporter::boolean::Boolean,
};

use necsim_impls_cuda::event_buffer::{Array, EventBuffer, EventType};
use necsim_impls_no_std::cogs::{
    active_lineage_sampler::singular::SingularActiveLineageSampler,
    event_sampler::tracking::MinSpeciationTrackingEventSampler,
};

use rust_cuda::{
    common::RustToCuda,
    host::{LaunchConfig, LaunchPackage, Launcher, TypedKernel},
    rustacuda::{
        error::{CudaError, CudaResult},
        function::{BlockSize, Function, GridSize},
    },
};

use rustcoalescence_algorithms_cuda_gpu_kernel::{
    BitonicGlobalSortSteppableKernel, BitonicSharedSortPreparableKernel,
    BitonicSharedSortSteppableKernel, EvenOddSortableKernel, SimulatableKernel,
};

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
        let kernel = Self::new_kernel()?;

        Ok(Self {
            kernel,
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
            watcher: &mut self.watcher,
        }
    }

    fn on_compile(kernel: &Function, watcher: &mut Self::CompilationWatcher) -> CudaResult<()> {
        (watcher)(kernel)
    }
}

pub struct EvenOddSortKernel<ReportSpeciation: Boolean, ReportDispersal: Boolean> {
    #[allow(clippy::type_complexity)]
    kernel: TypedKernel<dyn EvenOddSortableKernel<ReportSpeciation, ReportDispersal>>,
    grid: GridSize,
    block: BlockSize,
    ptx_jit: bool,
    watcher: Box<KernelCompilationCallback>,
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean>
    EvenOddSortKernel<ReportSpeciation, ReportDispersal>
{
    /// # Errors
    ///
    /// Returns a `CudaError` if loading the CUDA kernel failed.
    pub fn try_new(
        grid: GridSize,
        block: BlockSize,
        ptx_jit: bool,
        on_compile: Box<KernelCompilationCallback>,
    ) -> CudaResult<Self>
    where
        Self: EvenOddSortableKernel<ReportSpeciation, ReportDispersal>,
    {
        let kernel = Self::new_kernel()?;

        Ok(Self {
            kernel,
            grid,
            block,
            ptx_jit,
            watcher: on_compile,
        })
    }

    pub fn with_grid(&mut self, grid: GridSize) -> &mut Self {
        self.grid = grid;
        self
    }
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean> Launcher
    for EvenOddSortKernel<ReportSpeciation, ReportDispersal>
{
    type CompilationWatcher = Box<KernelCompilationCallback>;
    type KernelTraitObject = dyn EvenOddSortableKernel<ReportSpeciation, ReportDispersal>;

    fn get_launch_package(&mut self) -> LaunchPackage<Self> {
        LaunchPackage {
            config: LaunchConfig {
                grid: self.grid.clone(),
                block: self.block.clone(),
                shared_memory_size: 0_u32,
                ptx_jit: self.ptx_jit,
            },
            kernel: &mut self.kernel,
            watcher: &mut self.watcher,
        }
    }

    fn on_compile(kernel: &Function, watcher: &mut Self::CompilationWatcher) -> CudaResult<()> {
        (watcher)(kernel)
    }
}

pub struct BitonicGlobalSortStepKernel<ReportSpeciation: Boolean, ReportDispersal: Boolean> {
    #[allow(clippy::type_complexity)]
    kernel: TypedKernel<dyn BitonicGlobalSortSteppableKernel<ReportSpeciation, ReportDispersal>>,
    grid: GridSize,
    block: BlockSize,
    ptx_jit: bool,
    watcher: Box<KernelCompilationCallback>,
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean>
    BitonicGlobalSortStepKernel<ReportSpeciation, ReportDispersal>
{
    /// # Errors
    ///
    /// Returns a `CudaError` if loading the CUDA kernel failed.
    pub fn try_new(
        grid: GridSize,
        block: BlockSize,
        ptx_jit: bool,
        on_compile: Box<KernelCompilationCallback>,
    ) -> CudaResult<Self>
    where
        Self: BitonicGlobalSortSteppableKernel<ReportSpeciation, ReportDispersal>,
    {
        let kernel = Self::new_kernel()?;

        Ok(Self {
            kernel,
            grid,
            block,
            ptx_jit,
            watcher: on_compile,
        })
    }

    pub fn with_grid(&mut self, grid: GridSize) -> &mut Self {
        self.grid = grid;
        self
    }
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean> Launcher
    for BitonicGlobalSortStepKernel<ReportSpeciation, ReportDispersal>
{
    type CompilationWatcher = Box<KernelCompilationCallback>;
    type KernelTraitObject =
        dyn BitonicGlobalSortSteppableKernel<ReportSpeciation, ReportDispersal>;

    fn get_launch_package(&mut self) -> LaunchPackage<Self> {
        LaunchPackage {
            config: LaunchConfig {
                grid: self.grid.clone(),
                block: self.block.clone(),
                shared_memory_size: 0_u32,
                ptx_jit: self.ptx_jit,
            },
            kernel: &mut self.kernel,
            watcher: &mut self.watcher,
        }
    }

    fn on_compile(kernel: &Function, watcher: &mut Self::CompilationWatcher) -> CudaResult<()> {
        (watcher)(kernel)
    }
}

pub struct BitonicSharedSortStepKernel<ReportSpeciation: Boolean, ReportDispersal: Boolean> {
    #[allow(clippy::type_complexity)]
    kernel: TypedKernel<dyn BitonicSharedSortSteppableKernel<ReportSpeciation, ReportDispersal>>,
    grid: GridSize,
    block: BlockSize,
    ptx_jit: bool,
    watcher: Box<KernelCompilationCallback>,
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean>
    BitonicSharedSortStepKernel<ReportSpeciation, ReportDispersal>
{
    /// # Errors
    ///
    /// Returns a `CudaError` if loading the CUDA kernel failed.
    pub fn try_new(
        grid: GridSize,
        ptx_jit: bool,
        on_compile: Box<KernelCompilationCallback>,
    ) -> CudaResult<Self>
    where
        Self: BitonicSharedSortSteppableKernel<ReportSpeciation, ReportDispersal>,
    {
        let kernel = Self::new_kernel()?;

        let block_size = u32::try_from(
            <EventBuffer<ReportSpeciation, ReportDispersal> as EventType>::SharedBuffer::<()>::len(
            ) / 2,
        )
        .map_err(|_| CudaError::UnsupportedLimit)?;

        Ok(Self {
            kernel,
            grid,
            block: BlockSize::x(block_size),
            ptx_jit,
            watcher: on_compile,
        })
    }

    pub fn with_grid(&mut self, grid: GridSize) -> &mut Self {
        self.grid = grid;
        self
    }
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean> Launcher
    for BitonicSharedSortStepKernel<ReportSpeciation, ReportDispersal>
{
    type CompilationWatcher = Box<KernelCompilationCallback>;
    type KernelTraitObject =
        dyn BitonicSharedSortSteppableKernel<ReportSpeciation, ReportDispersal>;

    fn get_launch_package(&mut self) -> LaunchPackage<Self> {
        LaunchPackage {
            config: LaunchConfig {
                grid: self.grid.clone(),
                block: self.block.clone(),
                shared_memory_size: 0_u32,
                ptx_jit: self.ptx_jit,
            },
            kernel: &mut self.kernel,
            watcher: &mut self.watcher,
        }
    }

    fn on_compile(kernel: &Function, watcher: &mut Self::CompilationWatcher) -> CudaResult<()> {
        (watcher)(kernel)
    }
}

pub struct BitonicSharedSortPrepKernel<ReportSpeciation: Boolean, ReportDispersal: Boolean> {
    #[allow(clippy::type_complexity)]
    kernel: TypedKernel<dyn BitonicSharedSortPreparableKernel<ReportSpeciation, ReportDispersal>>,
    grid: GridSize,
    block: BlockSize,
    ptx_jit: bool,
    watcher: Box<KernelCompilationCallback>,
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean>
    BitonicSharedSortPrepKernel<ReportSpeciation, ReportDispersal>
{
    /// # Errors
    ///
    /// Returns a `CudaError` if loading the CUDA kernel failed.
    pub fn try_new(
        grid: GridSize,
        ptx_jit: bool,
        on_compile: Box<KernelCompilationCallback>,
    ) -> CudaResult<Self>
    where
        Self: BitonicSharedSortPreparableKernel<ReportSpeciation, ReportDispersal>,
    {
        let kernel = Self::new_kernel()?;

        let block_size = u32::try_from(
            <EventBuffer<ReportSpeciation, ReportDispersal> as EventType>::SharedBuffer::<()>::len(
            ) / 2,
        )
        .map_err(|_| CudaError::UnsupportedLimit)?;

        Ok(Self {
            kernel,
            grid,
            block: BlockSize::x(block_size),
            ptx_jit,
            watcher: on_compile,
        })
    }

    pub fn with_grid(&mut self, grid: GridSize) -> &mut Self {
        self.grid = grid;
        self
    }
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean> Launcher
    for BitonicSharedSortPrepKernel<ReportSpeciation, ReportDispersal>
{
    type CompilationWatcher = Box<KernelCompilationCallback>;
    type KernelTraitObject =
        dyn BitonicSharedSortPreparableKernel<ReportSpeciation, ReportDispersal>;

    fn get_launch_package(&mut self) -> LaunchPackage<Self> {
        LaunchPackage {
            config: LaunchConfig {
                grid: self.grid.clone(),
                block: self.block.clone(),
                shared_memory_size: 0_u32,
                ptx_jit: self.ptx_jit,
            },
            kernel: &mut self.kernel,
            watcher: &mut self.watcher,
        }
    }

    fn on_compile(kernel: &Function, watcher: &mut Self::CompilationWatcher) -> CudaResult<()> {
        (watcher)(kernel)
    }
}
