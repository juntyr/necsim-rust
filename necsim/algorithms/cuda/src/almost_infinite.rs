use necsim_core::lineage::Lineage;

use necsim_impls_no_std::cogs::{
    dispersal_sampler::almost_infinite_normal::AlmostInfiniteNormalDispersalSampler,
    habitat::almost_infinite::AlmostInfiniteHabitat,
    origin_sampler::{almost_infinite::AlmostInfiniteOriginSampler, pre_sampler::OriginPreSampler},
};

use necsim_impls_no_std::{
    partitioning::LocalPartition, reporter::ReporterContext,
    simulation::almost_infinite::AlmostInfiniteSimulation,
};

use super::{CudaArguments, CudaSimulation};

#[contract_trait]
impl AlmostInfiniteSimulation for CudaSimulation {
    type AuxiliaryArguments = CudaArguments;
    type Error = anyhow::Error;

    /// Simulates the coalescence algorithm on a CUDA-capable GPU on an
    /// almost-infinite `habitat` with N(0, sigma) `dispersal`. Only a
    /// circular region with `radius` is sampled.
    fn simulate<R: ReporterContext, P: LocalPartition<R>>(
        radius: u32,
        sigma: f64,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        local_partition: &mut P,
        auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = AlmostInfiniteHabitat::default();
        let dispersal_sampler = AlmostInfiniteNormalDispersalSampler::new(sigma);

        let lineages = AlmostInfiniteOriginSampler::new(
            OriginPreSampler::all().percentage(sample_percentage),
            &habitat,
            radius,
        )
        .map(|indexed_location| Lineage::new(indexed_location, &habitat))
        .collect();

        CudaSimulation::simulate(
            habitat,
            dispersal_sampler,
            lineages,
            speciation_probability_per_generation,
            seed,
            local_partition,
            auxiliary,
        )
    }
}
