use necsim_core::lineage::Lineage;

use necsim_impls_no_std::cogs::{
    dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
    habitat::non_spatial::NonSpatialHabitat,
    origin_sampler::{non_spatial::NonSpatialOriginSampler, percentage::PercentageOriginSampler},
};

use necsim_impls_no_std::{
    partitioning::Partitioning, reporter::ReporterContext,
    simulation::non_spatial::NonSpatialSimulation,
};

use super::{CudaArguments, CudaSimulation};

#[contract_trait]
impl NonSpatialSimulation for CudaSimulation {
    type AuxiliaryArguments = CudaArguments;
    type Error = anyhow::Error;

    /// Simulates the coalescence algorithm on a CUDA-capable GPU on a
    /// non-spatial `habitat` with non-spatial `dispersal`.
    fn simulate<P: Partitioning, R: ReporterContext>(
        area: (u32, u32),
        deme: u32,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        _partitioning: &mut P,
        reporter_context: R,
        auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = NonSpatialHabitat::new(area, deme);
        let dispersal_sampler = NonSpatialDispersalSampler::default();

        let lineages = PercentageOriginSampler::<NonSpatialHabitat>::new(
            NonSpatialOriginSampler::new(&habitat),
            sample_percentage,
        )
        .map(|indexed_location| Lineage::new(indexed_location, &habitat))
        .collect();

        CudaSimulation::simulate(
            habitat,
            dispersal_sampler,
            lineages,
            speciation_probability_per_generation,
            seed,
            reporter_context,
            &auxiliary,
        )
    }
}
