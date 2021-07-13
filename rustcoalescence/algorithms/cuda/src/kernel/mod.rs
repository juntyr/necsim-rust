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

// struct DummyLauncher;
//
// #[rust_cuda::host::kernel(use link_kernel! as impl Kernel for DummyLauncher)]
// pub fn simulate<
// H: Habitat + RustToCuda,
// G: PrimeableRng + RustToCuda,
// R: LineageReference<H> + DeviceCopy,
// S: LineageStore<H, R> + RustToCuda,
// X: EmigrationExit<H, G, R, S> + RustToCuda,
// D: DispersalSampler<H, G> + RustToCuda,
// C: CoalescenceSampler<H, R, S> + RustToCuda,
// T: TurnoverRate<H> + RustToCuda,
// N: SpeciationProbability<H> + RustToCuda,
// E: MinSpeciationTrackingEventSampler<H, G, R, S, X, D, C, T, N> + RustToCuda,
// I: ImmigrationEntry + RustToCuda,
// A: SingularActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
// + necsim_core::cogs::PeekableActiveLineageSampler<H, G, R, S, X, D, C, T, N,
// E, I> + RustToCuda,
// >
// (
// _simulation: &mut necsim_core::simulation::Simulation<H, G, R, S, X, D, C, T,
// N, E, I, A>, ) {
//
// }
//
// link_kernel!(
// necsim_impls_no_std::cogs::habitat::non_spatial::NonSpatialHabitat,
// necsim_impls_cuda::cogs::rng::CudaRng<necsim_impls_no_std::cogs::rng::wyhash:
// :WyHash>, necsim_core::lineage::GlobalLineageReference,
// necsim_impls_no_std::cogs::lineage_store::independent::
// IndependentLineageStore< necsim_impls_no_std::cogs::habitat::non_spatial::
// NonSpatialHabitat >,
// necsim_impls_no_std::cogs::emigration_exit::never::NeverEmigrationExit,
// necsim_impls_no_std::cogs::dispersal_sampler::non_spatial::
// NonSpatialDispersalSampler< necsim_impls_cuda::cogs::rng::
// CudaRng<necsim_impls_no_std::cogs::rng::wyhash::WyHash>, >,
// necsim_impls_no_std::cogs::coalescence_sampler::independent::
// IndependentCoalescenceSampler< necsim_impls_no_std::cogs::habitat::
// non_spatial::NonSpatialHabitat, >,
// necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
// necsim_impls_no_std::cogs::speciation_probability::uniform::
// UniformSpeciationProbability, necsim_impls_no_std::cogs::event_sampler::
// independent::IndependentEventSampler< necsim_impls_no_std::cogs::habitat::
// non_spatial::NonSpatialHabitat, necsim_impls_cuda::cogs::rng::
// CudaRng<necsim_impls_no_std::cogs::rng::wyhash::WyHash>, necsim_impls_no_std:
// :cogs::emigration_exit::never::NeverEmigrationExit, necsim_impls_no_std::
// cogs::dispersal_sampler::non_spatial::NonSpatialDispersalSampler<
// necsim_impls_cuda::cogs::rng::CudaRng<necsim_impls_no_std::cogs::rng::wyhash:
// :WyHash>, >,
// necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
// necsim_impls_no_std::cogs::speciation_probability::uniform::
// UniformSpeciationProbability, >,
// necsim_impls_no_std::cogs::immigration_entry::never::NeverImmigrationEntry,
// necsim_impls_no_std::cogs::active_lineage_sampler::independent::
// IndependentActiveLineageSampler< necsim_impls_no_std::cogs::habitat::
// non_spatial::NonSpatialHabitat, necsim_impls_cuda::cogs::rng::
// CudaRng<necsim_impls_no_std::cogs::rng::wyhash::WyHash>, necsim_impls_no_std:
// :cogs::emigration_exit::never::NeverEmigrationExit, necsim_impls_no_std::
// cogs::dispersal_sampler::non_spatial::NonSpatialDispersalSampler<
// necsim_impls_cuda::cogs::rng::CudaRng<necsim_impls_no_std::cogs::rng::wyhash:
// :WyHash>, >,
// necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
// necsim_impls_no_std::cogs::speciation_probability::uniform::
// UniformSpeciationProbability, necsim_impls_no_std::cogs::
// active_lineage_sampler::independent::event_time_sampler::exp::
// ExpEventTimeSampler, >,
// );
