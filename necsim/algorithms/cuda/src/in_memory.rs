use array2d::Array2D;

use necsim_core::lineage::Lineage;

use necsim_impls_no_std::cogs::{
    dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler,
    habitat::in_memory::InMemoryHabitat,
    origin_sampler::{in_memory::InMemoryOriginSampler, percentage::PercentageOriginSampler},
};
use necsim_impls_std::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;

use necsim_impls_no_std::{
    partitioning::Partitioning, reporter::ReporterContext,
    simulation::in_memory::InMemorySimulation,
};

use super::{CudaArguments, CudaSimulation};

#[contract_trait]
impl InMemorySimulation for CudaSimulation {
    type AuxiliaryArguments = CudaArguments;
    type Error = anyhow::Error;

    /// Simulates the coalescence algorithm on a CUDA-capable GPU on an in
    /// memory `habitat` with precalculated `dispersal`.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=RxC` where `habitat` has dimension
    /// `RxC`.
    fn simulate<P: Partitioning, R: ReporterContext>(
        habitat: &Array2D<u32>,
        dispersal: &Array2D<f64>,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        _partitioning: &mut P,
        reporter_context: R,
        auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = InMemoryHabitat::new(habitat.clone());
        let dispersal_sampler = InMemoryPackedAliasDispersalSampler::new(dispersal, &habitat)?;

        let lineages = PercentageOriginSampler::<InMemoryHabitat>::new(
            InMemoryOriginSampler::new(&habitat),
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
