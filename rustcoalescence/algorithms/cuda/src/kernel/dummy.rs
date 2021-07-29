use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageReference, LineageStore, MinSpeciationTrackingEventSampler,
        PeekableActiveLineageSampler, PrimeableRng, SingularActiveLineageSampler,
        SpeciationProbability, TurnoverRate,
    },
    reporter::boolean::Boolean,
};

use rust_cuda::{common::RustToCuda, rustacuda_core::DeviceCopy};

use rustcoalescence_algorithms_cuda_kernel::{Kernel, KernelArgs};

#[allow(clippy::type_complexity, clippy::module_name_repetitions)]
pub struct DummyLauncher<
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
    kernel: rust_cuda::host::TypedKernel<
        dyn Kernel<H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>,
    >,
    stream: rust_cuda::rustacuda::stream::Stream,
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
    > rust_cuda::host::Launcher
    for DummyLauncher<H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>
{
    type KernelTraitObject =
        dyn Kernel<H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>;

    fn get_config(&mut self) -> rust_cuda::host::LaunchConfig<Self::KernelTraitObject> {
        rust_cuda::host::LaunchConfig {
            stream: &mut self.stream,
            grid: 1.into(),
            block: 1.into(),
            shared_memory_size: 0_u32,
            kernel: &mut self.kernel,
        }
    }
}

rustcoalescence_algorithms_cuda_kernel::link_kernel!(
    necsim_impls_no_std::cogs::habitat::non_spatial::NonSpatialHabitat,
    necsim_impls_cuda::cogs::rng::CudaRng<necsim_impls_no_std::cogs::rng::wyhash::WyHash>,
    necsim_core::lineage::GlobalLineageReference,
    necsim_impls_no_std::cogs::lineage_store::independent::IndependentLineageStore<
        necsim_impls_no_std::cogs::habitat::non_spatial::NonSpatialHabitat
    >,
    necsim_impls_no_std::cogs::emigration_exit::never::NeverEmigrationExit,
    necsim_impls_no_std::cogs::dispersal_sampler::non_spatial::NonSpatialDispersalSampler<
        necsim_impls_cuda::cogs::rng::CudaRng<necsim_impls_no_std::cogs::rng::wyhash::WyHash>,
    >,
    necsim_impls_no_std::cogs::coalescence_sampler::independent::IndependentCoalescenceSampler<
        necsim_impls_no_std::cogs::habitat::non_spatial::NonSpatialHabitat,
    >,
    necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::uniform::UniformSpeciationProbability,
    necsim_impls_no_std::cogs::event_sampler::independent::IndependentEventSampler<
        necsim_impls_no_std::cogs::habitat::non_spatial::NonSpatialHabitat,
        necsim_impls_cuda::cogs::rng::CudaRng<necsim_impls_no_std::cogs::rng::wyhash::WyHash>,
        necsim_impls_no_std::cogs::emigration_exit::never::NeverEmigrationExit,
        necsim_impls_no_std::cogs::dispersal_sampler::non_spatial::NonSpatialDispersalSampler<
            necsim_impls_cuda::cogs::rng::CudaRng<necsim_impls_no_std::cogs::rng::wyhash::WyHash>,
        >,
        necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
        necsim_impls_no_std::cogs::speciation_probability::uniform::UniformSpeciationProbability,
    >,
    necsim_impls_no_std::cogs::immigration_entry::never::NeverImmigrationEntry,
    necsim_impls_no_std::cogs::active_lineage_sampler::independent::IndependentActiveLineageSampler<
        necsim_impls_no_std::cogs::habitat::non_spatial::NonSpatialHabitat,
        necsim_impls_cuda::cogs::rng::CudaRng<necsim_impls_no_std::cogs::rng::wyhash::WyHash>,
        necsim_impls_no_std::cogs::emigration_exit::never::NeverEmigrationExit,
        necsim_impls_no_std::cogs::dispersal_sampler::non_spatial::NonSpatialDispersalSampler<
            necsim_impls_cuda::cogs::rng::CudaRng<necsim_impls_no_std::cogs::rng::wyhash::WyHash>,
        >,
        necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
        necsim_impls_no_std::cogs::speciation_probability::uniform::UniformSpeciationProbability,
        necsim_impls_no_std::cogs::active_lineage_sampler::independent::event_time_sampler::exp::ExpEventTimeSampler,
    >,
    necsim_core::reporter::boolean::True,
    necsim_core::reporter::boolean::True,
);
