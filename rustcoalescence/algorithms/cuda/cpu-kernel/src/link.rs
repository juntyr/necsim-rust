use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageStore, MathsCore, PrimeableRng, SpeciationProbability, TurnoverRate,
    },
    reporter::boolean::Boolean,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::singular::SingularActiveLineageSampler,
    event_sampler::tracking::MinSpeciationTrackingEventSampler,
};

use rust_cuda::{
    common::RustToCuda,
    host::{LaunchConfig, LaunchPackage, Launcher},
    rustacuda::{error::CudaResult, function::Function},
};

#[allow(unused_imports)]
use rustcoalescence_algorithms_cuda_gpu_kernel::{SimulatableKernel, SimulationKernelArgs};

#[repr(transparent)]
pub struct SimulationKernel<
    M: MathsCore,
    H: Habitat<M> + RustToCuda,
    G: PrimeableRng<M> + RustToCuda,
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
>(
    #[allow(clippy::type_complexity)]
    pub(crate)  crate::SimulationKernel<
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
);

impl<
        M: MathsCore,
        H: Habitat<M> + RustToCuda,
        G: PrimeableRng<M> + RustToCuda,
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
    type CompilationWatcher = Box<crate::KernelCompilationCallback>;
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
                grid: self.0.grid.clone(),
                block: self.0.block.clone(),
                shared_memory_size: 0_u32,
                ptx_jit: self.0.ptx_jit,
            },

            kernel: &mut self.0.kernel,
            stream: &mut self.0.stream,

            watcher: &mut self.0.watcher,
        }
    }

    fn on_compile(kernel: &Function, watcher: &mut Self::CompilationWatcher) -> CudaResult<()> {
        (watcher)(kernel)
    }
}

#[allow(unused_macros)]
macro_rules! link_kernel {
    ($habitat:ty, $dispersal:ty, $turnover:ty, $speciation:ty) => {
        link_kernel! {
            $habitat, $dispersal, $turnover, $speciation,
            necsim_core::reporter::boolean::False,
            necsim_core::reporter::boolean::False
        }
        link_kernel! {
            $habitat, $dispersal, $turnover, $speciation,
            necsim_core::reporter::boolean::False,
            necsim_core::reporter::boolean::True
        }
        link_kernel! {
            $habitat, $dispersal, $turnover, $speciation,
            necsim_core::reporter::boolean::True,
            necsim_core::reporter::boolean::False
        }
        link_kernel! {
            $habitat, $dispersal, $turnover, $speciation,
            necsim_core::reporter::boolean::True,
            necsim_core::reporter::boolean::True
        }
    };
    (
        $habitat:ty, $dispersal:ty, $turnover:ty, $speciation:ty,
        $report_speciation:ty, $report_dispersal:ty
    ) => {
        rustcoalescence_algorithms_cuda_gpu_kernel::link_kernel!(
            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
            $habitat,
            necsim_impls_cuda::cogs::rng::CudaRng<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                    necsim_impls_cuda::cogs::maths::NvptxMathsCore
                >,
            >,
            necsim_impls_no_std::cogs::lineage_store::independent::IndependentLineageStore<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                $habitat,
            >,
            necsim_impls_no_std::cogs::emigration_exit::never::NeverEmigrationExit,
            $dispersal,
            necsim_impls_no_std::cogs::coalescence_sampler::independent::IndependentCoalescenceSampler<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                $habitat,
            >,
            $turnover,
            $speciation,
            necsim_impls_no_std::cogs::event_sampler::independent::IndependentEventSampler<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                $habitat,
                necsim_impls_cuda::cogs::rng::CudaRng<
                    necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    >,
                >,
                necsim_impls_no_std::cogs::emigration_exit::never::NeverEmigrationExit,
                $dispersal,
                $turnover,
                $speciation,
            >,
            necsim_impls_no_std::cogs::immigration_entry::never::NeverImmigrationEntry,
            necsim_impls_no_std::cogs::active_lineage_sampler::independent::IndependentActiveLineageSampler<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                $habitat,
                necsim_impls_cuda::cogs::rng::CudaRng<
                    necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    >,
                >,
                necsim_impls_no_std::cogs::emigration_exit::never::NeverEmigrationExit,
                $dispersal,
                $turnover,
                $speciation,
                necsim_impls_no_std::cogs::active_lineage_sampler::independent::event_time_sampler::exp::ExpEventTimeSampler,
            >,
            $report_speciation,
            $report_dispersal,
        );

        rustcoalescence_algorithms_cuda_gpu_kernel::link_kernel!(
            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
            $habitat,
            necsim_impls_cuda::cogs::rng::CudaRng<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                    necsim_impls_cuda::cogs::maths::NvptxMathsCore
                >,
            >,
            necsim_impls_no_std::cogs::lineage_store::independent::IndependentLineageStore<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                $habitat,
            >,
            necsim_impls_no_std::cogs::emigration_exit::never::NeverEmigrationExit,
            necsim_impls_no_std::cogs::dispersal_sampler::trespassing::TrespassingDispersalSampler<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                $habitat,
                necsim_impls_cuda::cogs::rng::CudaRng<
                    necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore
                    >,
                >,
                $dispersal,
                necsim_impls_no_std::cogs::dispersal_sampler::trespassing::uniform::UniformAntiTrespassingDispersalSampler<
                    necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    $habitat,
                    necsim_impls_cuda::cogs::rng::CudaRng<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                        necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                            necsim_impls_cuda::cogs::maths::NvptxMathsCore
                        >,
                    >,
                >,
            >,
            necsim_impls_no_std::cogs::coalescence_sampler::independent::IndependentCoalescenceSampler<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                $habitat,
            >,
            $turnover,
            $speciation,
            necsim_impls_no_std::cogs::event_sampler::independent::IndependentEventSampler<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                $habitat,
                necsim_impls_cuda::cogs::rng::CudaRng<
                    necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    >,
                >,
                necsim_impls_no_std::cogs::emigration_exit::never::NeverEmigrationExit,
                necsim_impls_no_std::cogs::dispersal_sampler::trespassing::TrespassingDispersalSampler<
                    necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    $habitat,
                    necsim_impls_cuda::cogs::rng::CudaRng<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                        necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                            necsim_impls_cuda::cogs::maths::NvptxMathsCore
                        >,
                    >,
                    $dispersal,
                    necsim_impls_no_std::cogs::dispersal_sampler::trespassing::uniform::UniformAntiTrespassingDispersalSampler<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                        $habitat,
                        necsim_impls_cuda::cogs::rng::CudaRng<
                            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                            necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                                necsim_impls_cuda::cogs::maths::NvptxMathsCore
                            >,
                        >,
                    >,
                >,
                $turnover,
                $speciation,
            >,
            necsim_impls_no_std::cogs::immigration_entry::never::NeverImmigrationEntry,
            necsim_impls_no_std::cogs::active_lineage_sampler::independent::IndependentActiveLineageSampler<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                $habitat,
                necsim_impls_cuda::cogs::rng::CudaRng<
                    necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    >,
                >,
                necsim_impls_no_std::cogs::emigration_exit::never::NeverEmigrationExit,
                necsim_impls_no_std::cogs::dispersal_sampler::trespassing::TrespassingDispersalSampler<
                    necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    $habitat,
                    necsim_impls_cuda::cogs::rng::CudaRng<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                        necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                            necsim_impls_cuda::cogs::maths::NvptxMathsCore
                        >,
                    >,
                    $dispersal,
                    necsim_impls_no_std::cogs::dispersal_sampler::trespassing::uniform::UniformAntiTrespassingDispersalSampler<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                        $habitat,
                        necsim_impls_cuda::cogs::rng::CudaRng<
                            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                            necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                                necsim_impls_cuda::cogs::maths::NvptxMathsCore
                            >,
                        >,
                    >,
                >,
                $turnover,
                $speciation,
                necsim_impls_no_std::cogs::active_lineage_sampler::independent::event_time_sampler::r#const::ConstEventTimeSampler,
            >,
            $report_speciation,
            $report_dispersal,
        );
    };
}

#[cfg(feature = "non-spatial-scenario")]
link_kernel!(
    necsim_impls_no_std::cogs::habitat::non_spatial::NonSpatialHabitat<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore
    >,
    necsim_impls_no_std::cogs::dispersal_sampler::non_spatial::NonSpatialDispersalSampler<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
        necsim_impls_cuda::cogs::rng::CudaRng<
            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
            necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore
            >,
        >,
    >,
    necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::uniform::UniformSpeciationProbability
);

#[cfg(feature = "spatially-implicit-scenario")]
link_kernel!(
    necsim_impls_no_std::cogs::habitat::spatially_implicit::SpatiallyImplicitHabitat<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore
    >,
    necsim_impls_no_std::cogs::dispersal_sampler::spatially_implicit::SpatiallyImplicitDispersalSampler<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
        necsim_impls_cuda::cogs::rng::CudaRng<
            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
            necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore
            >,
        >,
    >,
    necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::spatially_implicit::SpatiallyImplicitSpeciationProbability
);

#[cfg(feature = "almost-infinite-normal-dispersal-scenario")]
link_kernel!(
    necsim_impls_no_std::cogs::habitat::almost_infinite::AlmostInfiniteHabitat<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore
    >,
    necsim_impls_no_std::cogs::dispersal_sampler::almost_infinite_normal::AlmostInfiniteNormalDispersalSampler<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
        necsim_impls_cuda::cogs::rng::CudaRng<
            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
            necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore
            >,
        >,
    >,
    necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::uniform::UniformSpeciationProbability
);

#[cfg(feature = "almost-infinite-clark2dt-dispersal-scenario")]
link_kernel!(
    necsim_impls_no_std::cogs::habitat::almost_infinite::AlmostInfiniteHabitat<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore
    >,
    necsim_impls_no_std::cogs::dispersal_sampler::almost_infinite_clark2dt::AlmostInfiniteClark2DtDispersalSampler<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
        necsim_impls_cuda::cogs::rng::CudaRng<
            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
            necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore
            >,
        >,
    >,
    necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::uniform::UniformSpeciationProbability
);

#[cfg(feature = "spatially-explicit-uniform-turnover-scenario")]
link_kernel!(
    necsim_impls_no_std::cogs::habitat::in_memory::InMemoryHabitat<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore
    >,
    necsim_impls_no_std::cogs::dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
        necsim_impls_no_std::cogs::habitat::in_memory::InMemoryHabitat<
            necsim_impls_cuda::cogs::maths::NvptxMathsCore
        >,
        necsim_impls_cuda::cogs::rng::CudaRng<
            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
            necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore
            >,
        >,
    >,
    necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::uniform::UniformSpeciationProbability
);

#[cfg(feature = "spatially-explicit-turnover-map-scenario")]
link_kernel!(
    necsim_impls_no_std::cogs::habitat::in_memory::InMemoryHabitat<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore
    >,
    necsim_impls_no_std::cogs::dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
        necsim_impls_no_std::cogs::habitat::in_memory::InMemoryHabitat<
            necsim_impls_cuda::cogs::maths::NvptxMathsCore
        >,
        necsim_impls_cuda::cogs::rng::CudaRng<
            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
            necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore
            >,
        >,
    >,
    necsim_impls_no_std::cogs::turnover_rate::in_memory::InMemoryTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::uniform::UniformSpeciationProbability
);

#[cfg(feature = "wrapping-noise-scenario")]
link_kernel!(
    necsim_impls_no_std::cogs::habitat::wrapping_noise::WrappingNoiseHabitat<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore
    >,
    necsim_impls_no_std::cogs::dispersal_sampler::wrapping_noise::WrappingNoiseApproximateNormalDispersalSampler<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
        necsim_impls_cuda::cogs::rng::CudaRng<
            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
            necsim_impls_no_std::cogs::rng::wyhash::WyHash<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore
            >,
        >,
    >,
    necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::uniform::UniformSpeciationProbability
);
