use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::almost_infinite_normal::AlmostInfiniteNormalDispersalSampler,
        habitat::almost_infinite::AlmostInfiniteHabitat,
        lineage_store::coherent::almost_infinite::CoherentAlmostInfiniteLineageStore,
        origin_sampler::{
            almost_infinite::AlmostInfiniteOriginSampler,
            decomposition::DecompositionOriginSampler, pre_sampler::OriginPreSampler,
        },
    },
    decomposition::radial::RadialDecomposition,
};

use necsim_impls_no_std::{
    partitioning::LocalPartition, reporter::ReporterContext,
    simulation::almost_infinite::AlmostInfiniteSimulation,
};

use super::{SkippingGillespieArguments, SkippingGillespieSimulation};

#[contract_trait]
impl AlmostInfiniteSimulation for SkippingGillespieSimulation {
    type AuxiliaryArguments = SkippingGillespieArguments;
    type Error = anyhow::Error;

    /// Simulates the Gillespie coalescence algorithm with self-dispersal event
    /// skipping on on an almost-infinite `habitat` with N(0, sigma)
    /// `dispersal`. Only a circular region with `radius` is sampled.
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

        let decomposition = RadialDecomposition::new(
            local_partition.get_partition_rank(),
            local_partition.get_number_of_partitions(),
        );

        let lineage_store = if local_partition.get_number_of_partitions().get() > 1 {
            CoherentAlmostInfiniteLineageStore::new(DecompositionOriginSampler::new(
                AlmostInfiniteOriginSampler::new(
                    OriginPreSampler::all().percentage(sample_percentage),
                    &habitat,
                    radius,
                ),
                &decomposition,
            ))
        } else {
            CoherentAlmostInfiniteLineageStore::new(AlmostInfiniteOriginSampler::new(
                OriginPreSampler::all().percentage(sample_percentage),
                &habitat,
                radius,
            ))
        };

        SkippingGillespieSimulation::simulate(
            habitat,
            dispersal_sampler,
            lineage_store,
            speciation_probability_per_generation,
            seed,
            local_partition,
            decomposition,
            auxiliary,
        )
    }
}
