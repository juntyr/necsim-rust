use rustcoalescence_algorithms_cuda_gpu_kernel::{SimulatableKernel, SimulationKernelArgs};

use crate::SimulationKernel;

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
                necsim_impls_no_std::cogs::rng::simple::SimpleRng<
                    necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    necsim_impls_no_std::cogs::rng::wyhash::WyHash,
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
                    necsim_impls_no_std::cogs::rng::simple::SimpleRng<
                    necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    necsim_impls_no_std::cogs::rng::wyhash::WyHash,
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
                    necsim_impls_no_std::cogs::rng::simple::SimpleRng<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                        necsim_impls_no_std::cogs::rng::wyhash::WyHash,
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
                necsim_impls_no_std::cogs::rng::simple::SimpleRng<
                    necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    necsim_impls_no_std::cogs::rng::wyhash::WyHash,
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
                    necsim_impls_no_std::cogs::rng::simple::SimpleRng<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                        necsim_impls_no_std::cogs::rng::wyhash::WyHash,
                    >,
                >,
                $dispersal,
                necsim_impls_no_std::cogs::dispersal_sampler::trespassing::uniform::UniformAntiTrespassingDispersalSampler<
                    necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    $habitat,
                    necsim_impls_cuda::cogs::rng::CudaRng<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                        necsim_impls_no_std::cogs::rng::simple::SimpleRng<
                            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                            necsim_impls_no_std::cogs::rng::wyhash::WyHash,
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
                    necsim_impls_no_std::cogs::rng::simple::SimpleRng<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                        necsim_impls_no_std::cogs::rng::wyhash::WyHash,
                    >,
                >,
                necsim_impls_no_std::cogs::emigration_exit::never::NeverEmigrationExit,
                necsim_impls_no_std::cogs::dispersal_sampler::trespassing::TrespassingDispersalSampler<
                    necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    $habitat,
                    necsim_impls_cuda::cogs::rng::CudaRng<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                        necsim_impls_no_std::cogs::rng::simple::SimpleRng<
                            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                            necsim_impls_no_std::cogs::rng::wyhash::WyHash,
                        >,
                    >,
                    $dispersal,
                    necsim_impls_no_std::cogs::dispersal_sampler::trespassing::uniform::UniformAntiTrespassingDispersalSampler<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                        $habitat,
                        necsim_impls_cuda::cogs::rng::CudaRng<
                            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                            necsim_impls_no_std::cogs::rng::simple::SimpleRng<
                                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                                necsim_impls_no_std::cogs::rng::wyhash::WyHash,
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
                    necsim_impls_no_std::cogs::rng::simple::SimpleRng<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                        necsim_impls_no_std::cogs::rng::wyhash::WyHash,
                    >,
                >,
                necsim_impls_no_std::cogs::emigration_exit::never::NeverEmigrationExit,
                necsim_impls_no_std::cogs::dispersal_sampler::trespassing::TrespassingDispersalSampler<
                    necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                    $habitat,
                    necsim_impls_cuda::cogs::rng::CudaRng<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                        necsim_impls_no_std::cogs::rng::simple::SimpleRng<
                            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                            necsim_impls_no_std::cogs::rng::wyhash::WyHash,
                        >,
                    >,
                    $dispersal,
                    necsim_impls_no_std::cogs::dispersal_sampler::trespassing::uniform::UniformAntiTrespassingDispersalSampler<
                        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                        $habitat,
                        necsim_impls_cuda::cogs::rng::CudaRng<
                            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                            necsim_impls_no_std::cogs::rng::simple::SimpleRng<
                                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                                necsim_impls_no_std::cogs::rng::wyhash::WyHash,
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

link_kernel!(
    necsim_impls_no_std::cogs::habitat::non_spatial::NonSpatialHabitat<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore
    >,
    necsim_impls_no_std::cogs::dispersal_sampler::non_spatial::NonSpatialDispersalSampler<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
        necsim_impls_cuda::cogs::rng::CudaRng<
            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
            necsim_impls_no_std::cogs::rng::simple::SimpleRng<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                necsim_impls_no_std::cogs::rng::wyhash::WyHash,
            >,
        >,
    >,
    necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::uniform::UniformSpeciationProbability
);

link_kernel!(
    necsim_impls_no_std::cogs::habitat::spatially_implicit::SpatiallyImplicitHabitat<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore
    >,
    necsim_impls_no_std::cogs::dispersal_sampler::spatially_implicit::SpatiallyImplicitDispersalSampler<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
        necsim_impls_cuda::cogs::rng::CudaRng<
            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
            necsim_impls_no_std::cogs::rng::simple::SimpleRng<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                necsim_impls_no_std::cogs::rng::wyhash::WyHash,
            >,
        >,
    >,
    necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::spatially_implicit::SpatiallyImplicitSpeciationProbability
);

link_kernel!(
    necsim_impls_no_std::cogs::habitat::almost_infinite::AlmostInfiniteHabitat<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore
    >,
    necsim_impls_no_std::cogs::dispersal_sampler::almost_infinite_normal::AlmostInfiniteNormalDispersalSampler<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
        necsim_impls_cuda::cogs::rng::CudaRng<
            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
            necsim_impls_no_std::cogs::rng::simple::SimpleRng<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                necsim_impls_no_std::cogs::rng::wyhash::WyHash,
            >,
        >,
    >,
    necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::uniform::UniformSpeciationProbability
);

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
            necsim_impls_no_std::cogs::rng::simple::SimpleRng<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                necsim_impls_no_std::cogs::rng::wyhash::WyHash,
            >,
        >,
    >,
    necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::uniform::UniformSpeciationProbability
);

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
            necsim_impls_no_std::cogs::rng::simple::SimpleRng<
                necsim_impls_cuda::cogs::maths::NvptxMathsCore,
                necsim_impls_no_std::cogs::rng::wyhash::WyHash,
            >,
        >,
    >,
    necsim_impls_no_std::cogs::turnover_rate::in_memory::InMemoryTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::uniform::UniformSpeciationProbability
);

link_kernel!(
    necsim_impls_no_std::cogs::habitat::wrapping_noise::WrappingNoiseHabitat<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore
    >,
    necsim_impls_no_std::cogs::dispersal_sampler::wrapping_noise::WrappingNoiseApproximateNormalDispersalSampler<
        necsim_impls_cuda::cogs::maths::NvptxMathsCore,
        necsim_impls_cuda::cogs::rng::CudaRng<
            necsim_impls_cuda::cogs::maths::NvptxMathsCore,
            necsim_impls_no_std::cogs::rng::wyhash::WyHash,
        >,
    >,
    necsim_impls_no_std::cogs::turnover_rate::uniform::UniformTurnoverRate,
    necsim_impls_no_std::cogs::speciation_probability::uniform::UniformSpeciationProbability
);
