#![deny(clippy::pedantic)]
#![no_std]
#![feature(abi_ptx)]
#![cfg_attr(target_os = "cuda", feature(alloc_error_handler))]
#![cfg_attr(target_os = "cuda", feature(panic_info_message))]
#![feature(atomic_from_mut)]
#![feature(asm)]
// TODO: Remove once https://github.com/rust-lang/rust/issues/87551 is fixed
#![feature(core_intrinsics)]
#![feature(const_type_id)]
#![allow(incomplete_features)]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)]

extern crate alloc;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageReference, LineageStore, MinSpeciationTrackingEventSampler,
        PeekableActiveLineageSampler, PrimeableRng, SingularActiveLineageSampler,
        SpeciationProbability, SpeciationSample, TurnoverRate,
    },
    reporter::boolean::Boolean,
};

use rust_cuda::{common::RustToCuda, rustacuda_core::DeviceCopy};

#[cfg(not(target_os = "cuda"))]
mod rustcoalescence_algorithms_cuda {
    pub mod kernel {
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

        #[allow(dead_code, clippy::type_complexity)]
        pub struct DummyLauncher<
            H: 'static + Habitat + RustToCuda,
            G: 'static + PrimeableRng + RustToCuda,
            R: 'static + LineageReference<H> + DeviceCopy,
            S: 'static + LineageStore<H, R> + RustToCuda,
            X: 'static + EmigrationExit<H, G, R, S> + RustToCuda,
            D: 'static + DispersalSampler<H, G> + RustToCuda,
            C: 'static + CoalescenceSampler<H, R, S> + RustToCuda,
            T: 'static + TurnoverRate<H> + RustToCuda,
            N: 'static + SpeciationProbability<H> + RustToCuda,
            E: 'static + MinSpeciationTrackingEventSampler<H, G, R, S, X, D, C, T, N> + RustToCuda,
            I: 'static + ImmigrationEntry + RustToCuda,
            A: 'static
                + SingularActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
                + PeekableActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
                + RustToCuda,
            ReportSpeciation: 'static + Boolean,
            ReportDispersal: 'static + Boolean,
        >
        where
            rust_cuda::host::TypedKernel<
                {
                    core::intrinsics::type_id::<
                        dyn crate::Kernel<
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
                        >,
                    >()
                },
            >: Sized,
        {
            kernel: rust_cuda::host::TypedKernel<
                {
                    core::intrinsics::type_id::<
                        dyn crate::Kernel<
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
                        >,
                    >()
                },
            >,
            _marker: core::marker::PhantomData<(
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
                C,
                I,
                A,
                ReportSpeciation,
                ReportDispersal,
            )>,
        }

        impl<
                H: 'static + Habitat + RustToCuda,
                G: 'static + PrimeableRng + RustToCuda,
                R: 'static + LineageReference<H> + DeviceCopy,
                S: 'static + LineageStore<H, R> + RustToCuda,
                X: 'static + EmigrationExit<H, G, R, S> + RustToCuda,
                D: 'static + DispersalSampler<H, G> + RustToCuda,
                C: 'static + CoalescenceSampler<H, R, S> + RustToCuda,
                T: 'static + TurnoverRate<H> + RustToCuda,
                N: 'static + SpeciationProbability<H> + RustToCuda,
                E: 'static + MinSpeciationTrackingEventSampler<H, G, R, S, X, D, C, T, N> + RustToCuda,
                I: 'static + ImmigrationEntry + RustToCuda,
                A: 'static
                    + SingularActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
                    + PeekableActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
                    + RustToCuda,
                ReportSpeciation: 'static + Boolean,
                ReportDispersal: 'static + Boolean,
            >
            rust_cuda::host::Launcher<
                {
                    core::intrinsics::type_id::<
                        dyn crate::Kernel<
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
                        >,
                    >()
                },
            >
            for DummyLauncher<H, G, R, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>
        {
            fn get_launch_params(
                &mut self,
            ) -> rust_cuda::host::LaunchParams<
                {
                    core::intrinsics::type_id::<
                        dyn crate::Kernel<
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
                        >,
                    >()
                },
            > {
                // rust_cuda::host::LaunchParams {
                // stream: &mut self.stream,
                // grid: 1.into(),
                // block: 1.into(),
                // shared_memory_size: 0_u32,
                // kernel: &mut self.kernel,
                // }

                unimplemented!()
            }
        }
    }
}

#[cfg(target_os = "cuda")]
mod cuda_prelude {
    use rust_cuda::device::{nvptx, utils};

    #[global_allocator]
    static _GLOBAL_ALLOCATOR: utils::PTXAllocator = utils::PTXAllocator;

    #[cfg(not(debug_assertions))]
    #[panic_handler]
    fn panic(_panic_info: &::core::panic::PanicInfo) -> ! {
        unsafe { nvptx::trap() }
    }

    #[cfg(debug_assertions)]
    #[panic_handler]
    fn panic(panic_info: &::core::panic::PanicInfo) -> ! {
        use rust_cuda::println;

        println!(
            "Panic occurred at {:?}: {:?}!",
            panic_info.location(),
            panic_info
                .message()
                .unwrap_or(&format_args!("unknown reason"))
        );

        unsafe { nvptx::trap() }
    }

    #[alloc_error_handler]
    fn alloc_error_handler(_: core::alloc::Layout) -> ! {
        unsafe { nvptx::trap() }
    }
}

#[rust_cuda::common::kernel(use link_kernel! as impl Kernel<KernelArgs> for rustcoalescence_algorithms_cuda::kernel::DummyLauncher)]
#[allow(clippy::too_many_arguments)]
pub fn simulate<
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
>(
    #[rustfmt::skip]
    #[kernel(pass = RustToCuda, jit)]
    simulation: &mut necsim_core::simulation::Simulation<H, G, R, S, X, D, C, T, N, E, I, A>,
    #[rustfmt::skip]
    #[kernel(pass = RustToCuda, jit)]
    task_list: &mut necsim_impls_cuda::value_buffer::ValueBuffer<necsim_core::lineage::Lineage>,
    #[rustfmt::skip]
    #[kernel(pass = RustToCuda, jit)]
    event_buffer_reporter: &mut necsim_impls_cuda::event_buffer::EventBuffer<
        ReportSpeciation,
        ReportDispersal,
    >,
    #[rustfmt::skip]
    #[kernel(pass = RustToCuda, jit)]
    min_spec_sample_buffer: &mut necsim_impls_cuda::value_buffer::ValueBuffer<SpeciationSample>,
    #[rustfmt::skip]
    #[kernel(pass = RustToCuda, jit)]
    next_event_time_buffer: &mut necsim_impls_cuda::value_buffer::ValueBuffer<
        necsim_core_bond::PositiveF64,
    >,
    #[rustfmt::skip]
    #[kernel(pass = DeviceCopy)]
    total_time_max: &mut u64,
    #[rustfmt::skip]
    #[kernel(pass = DeviceCopy)]
    total_steps_sum: &mut u64,
    #[rustfmt::skip]
    #[kernel(pass = DeviceCopy)]
    max_steps: u64,
    #[rustfmt::skip]
    #[kernel(pass = DeviceCopy)]
    max_next_event_time: necsim_core_bond::NonNegativeF64,
) {
    let total_time_max = core::sync::atomic::AtomicU64::from_mut(total_time_max);
    let total_steps_sum = core::sync::atomic::AtomicU64::from_mut(total_steps_sum);

    task_list.with_value_for_core(|task| {
        // Discard the prior task (the simulation is just a temporary local copy)
        core::mem::drop(
            simulation
                .active_lineage_sampler_mut()
                .replace_active_lineage(task),
        );

        min_spec_sample_buffer.with_value_for_core(|min_spec_sample| {
            // Discard the prior sample (same reason as above)
            simulation
                .event_sampler_mut()
                .replace_min_speciation(min_spec_sample);

            let (time, steps) = simulation.simulate_incremental_early_stop(
                |simulation, steps| {
                    steps >= max_steps
                        || simulation
                            .peek_time_of_next_event()
                            .map_or(true, |next_time| next_time >= max_next_event_time)
                },
                event_buffer_reporter,
            );

            next_event_time_buffer.with_value_for_core(|_| simulation.peek_time_of_next_event());

            if steps > 0 {
                total_time_max
                    .fetch_max(time.get().to_bits(), core::sync::atomic::Ordering::Relaxed);
                total_steps_sum.fetch_add(steps, core::sync::atomic::Ordering::Relaxed);
            }

            simulation.event_sampler_mut().replace_min_speciation(None)
        });

        simulation
            .active_lineage_sampler_mut()
            .replace_active_lineage(None)
    });
}

#[cfg(not(target_os = "cuda"))]
link_kernel!(
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
