use necsim_impls_no_std::cogs::{
    dispersal_sampler::almost_infinite_normal::AlmostInfiniteNormalDispersalSampler,
    habitat::almost_infinite::AlmostInfiniteHabitat,
    lineage_store::coherent::almost_infinite::CoherentAlmostInfiniteLineageStore,
    origin_sampler::{
        almost_infinite::AlmostInfiniteOriginSampler, percentage::PercentageOriginSampler,
    },
};

use necsim_impls_no_std::{
    partitioning::Partitioning, reporter::ReporterContext,
    simulation::almost_infinite::AlmostInfiniteSimulation,
};

use super::SkippingGillespieSimulation;

#[contract_trait]
impl AlmostInfiniteSimulation for SkippingGillespieSimulation {
    type AuxiliaryArguments = ();
    type Error = !;

    /// Simulates the Gillespie coalescence algorithm with self-dispersal event
    /// skipping on on an almost-infinite `habitat` with N(0, sigma)
    /// `dispersal`. Only a circular region with `radius` is sampled.
    fn simulate<P: Partitioning, R: ReporterContext>(
        radius: u32,
        sigma: f64,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        _partitioning: &mut P,
        reporter_context: R,
        _auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = AlmostInfiniteHabitat::default();
        let dispersal_sampler = AlmostInfiniteNormalDispersalSampler::new(sigma);
        let lineage_store = CoherentAlmostInfiniteLineageStore::new(PercentageOriginSampler::new(
            AlmostInfiniteOriginSampler::new(&habitat, radius),
            sample_percentage,
        ));

        Ok(SkippingGillespieSimulation::simulate(
            habitat,
            dispersal_sampler,
            lineage_store,
            speciation_probability_per_generation,
            seed,
            reporter_context,
        ))
    }
}
