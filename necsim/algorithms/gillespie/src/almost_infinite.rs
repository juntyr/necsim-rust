use necsim_impls_no_std::cogs::{
    dispersal_sampler::almost_infinite_normal::AlmostInfiniteNormalDispersalSampler,
    habitat::almost_infinite::AlmostInfiniteHabitat,
    lineage_store::coherent::globally::almost_infinite::AlmostInfiniteLineageStore,
    origin_sampler::{almost_infinite::AlmostInfiniteOriginSampler, pre_sampler::OriginPreSampler},
};

use necsim_impls_no_std::{
    partitioning::LocalPartition, reporter::ReporterContext,
    simulation::almost_infinite::AlmostInfiniteSimulation,
};

use super::GillespieSimulation;

#[contract_trait]
impl AlmostInfiniteSimulation for GillespieSimulation {
    type AuxiliaryArguments = ();
    type Error = !;

    /// Simulates the Gillespie coalescence algorithm on on an almost-infinite
    /// `habitat` with N(0, sigma) `dispersal`. Only a circular region with
    /// `radius` is sampled.
    fn simulate<R: ReporterContext, P: LocalPartition<R>>(
        radius: u32,
        sigma: f64,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        local_partition: &mut P,
        _auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = AlmostInfiniteHabitat::default();
        let dispersal_sampler = AlmostInfiniteNormalDispersalSampler::new(sigma);

        let lineage_store = AlmostInfiniteLineageStore::new(AlmostInfiniteOriginSampler::new(
            OriginPreSampler::all().percentage(sample_percentage),
            &habitat,
            radius,
        ));

        Ok(GillespieSimulation::simulate(
            habitat,
            dispersal_sampler,
            lineage_store,
            speciation_probability_per_generation,
            seed,
            local_partition,
        ))
    }
}
