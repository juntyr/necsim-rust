use necsim_core::lineage::Lineage;

use necsim_impls_no_std::cogs::{
    dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
    habitat::non_spatial::NonSpatialHabitat,
    origin_sampler::{non_spatial::NonSpatialOriginSampler, pre_sampler::OriginPreSampler},
};

use necsim_impls_no_std::{
    partitioning::LocalPartition, reporter::ReporterContext,
    simulation::non_spatial::NonSpatialSimulation,
};

use super::{CudaArguments, CudaSimulation};

#[contract_trait]
impl NonSpatialSimulation for CudaSimulation {
    type AuxiliaryArguments = CudaArguments;
    type Error = anyhow::Error;

    /// Simulates the coalescence algorithm on a CUDA-capable GPU on a
    /// non-spatial `habitat` with non-spatial `dispersal`.
    fn simulate<R: ReporterContext, P: LocalPartition<R>>(
        area: (u32, u32),
        deme: u32,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        local_partition: &mut P,
        auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = NonSpatialHabitat::new(area, deme);
        let dispersal_sampler = NonSpatialDispersalSampler::default();

        let lineages = NonSpatialOriginSampler::new(
            OriginPreSampler::all().percentage(sample_percentage),
            &habitat,
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
            &auxiliary,
        )
    }
}
