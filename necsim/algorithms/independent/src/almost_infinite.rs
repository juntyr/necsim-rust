use necsim_core::lineage::Lineage;

use necsim_impls_no_std::cogs::{
    dispersal_sampler::almost_infinite_normal::AlmostInfiniteNormalDispersalSampler,
    habitat::almost_infinite::AlmostInfiniteHabitat,
    origin_sampler::{
        almost_infinite::AlmostInfiniteOriginSampler, percentage::PercentageOriginSampler,
    },
    speciation_probability::uniform::UniformSpeciationProbability,
};

use necsim_impls_no_std::{
    reporter::ReporterContext, simulation::almost_infinite::AlmostInfiniteSimulation,
};

use super::IndependentSimulation;

#[contract_trait]
impl AlmostInfiniteSimulation for IndependentSimulation {
    type AuxiliaryArguments = ();
    type Error = !;

    /// Simulates the independent coalescence algorithm on an almost-infinite
    /// `habitat` with N(0, sigma) `dispersal`. Only a circular region with
    /// `radius` is sampled.
    fn simulate<P: ReporterContext>(
        radius: u32,
        sigma: f64,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        reporter_context: P,
        _auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = AlmostInfiniteHabitat::default();
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation);
        let dispersal_sampler = AlmostInfiniteNormalDispersalSampler::new(sigma);

        let lineages = PercentageOriginSampler::<AlmostInfiniteHabitat>::new(
            AlmostInfiniteOriginSampler::new(&habitat, radius),
            sample_percentage,
        )
        .map(|indexed_location| Lineage::new(indexed_location, &habitat))
        .collect();

        Ok(IndependentSimulation::simulate(
            habitat,
            speciation_probability,
            dispersal_sampler,
            lineages,
            seed,
            reporter_context,
        ))
    }
}
