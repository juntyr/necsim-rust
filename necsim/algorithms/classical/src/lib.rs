#![deny(clippy::pedantic)]
#![feature(never_type)]

#[macro_use]
extern crate contracts;

use necsim_core::{
    cogs::{
        DispersalSampler, Habitat, LineageReference, LocallyCoherentLineageStore, RngCore,
        SpeciationProbability,
    },
    simulation::Simulation,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::classical::ClassicalActiveLineageSampler,
    coalescence_sampler::unconditional::UnconditionalCoalescenceSampler,
    emigration_exit::never::NeverEmigrationExit,
    event_sampler::unconditional::UnconditionalEventSampler,
    immigration_entry::never::NeverImmigrationEntry, turnover_rate::uniform::UniformTurnoverRate,
};
use necsim_impls_std::cogs::rng::std::StdRng;

use necsim_impls_no_std::{partitioning::LocalPartition, reporter::ReporterContext};

mod almost_infinite;
mod in_memory;
mod non_spatial;
mod spatially_implicit;

pub struct ClassicalSimulation;

impl ClassicalSimulation {
    /// Simulates the classical coalescence algorithm on the `habitat` with
    /// `dispersal` and lineages from `lineage_store`.
    fn simulate_chain<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: LocallyCoherentLineageStore<H, R>,
        P: ReporterContext,
        L: LocalPartition<P>,
    >(
        habitat: H,
        speciation_probability: N,
        dispersal_sampler: D,
        lineage_store: S,
        rng: G,
        local_partition: &mut L,
    ) -> (f64, u64, G) {
        let coalescence_sampler = UnconditionalCoalescenceSampler::default();
        let emigration_exit = NeverEmigrationExit::default();
        let turnover_rate = UniformTurnoverRate::default();
        let event_sampler = UnconditionalEventSampler::default();
        let immigration_entry = NeverImmigrationEntry::default();
        let active_lineage_sampler = ClassicalActiveLineageSampler::new(&lineage_store);

        let simulation = Simulation::builder()
            .habitat(habitat)
            .rng(rng)
            .turnover_rate(turnover_rate)
            .speciation_probability(speciation_probability)
            .dispersal_sampler(dispersal_sampler)
            .lineage_reference(std::marker::PhantomData::<R>)
            .lineage_store(lineage_store)
            .emigration_exit(emigration_exit)
            .coalescence_sampler(coalescence_sampler)
            .event_sampler(event_sampler)
            .immigration_entry(immigration_entry)
            .active_lineage_sampler(active_lineage_sampler)
            .build();

        simulation.simulate(local_partition.get_reporter())
    }

    /// Simulates the classical coalescence algorithm on the `habitat` with
    /// `dispersal` and lineages from `lineage_store`.
    fn simulate<
        H: Habitat,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, StdRng>,
        R: LineageReference<H>,
        S: LocallyCoherentLineageStore<H, R>,
        P: ReporterContext,
        L: LocalPartition<P>,
    >(
        habitat: H,
        speciation_probability: N,
        dispersal_sampler: D,
        lineage_store: S,
        seed: u64,
        local_partition: &mut L,
    ) -> (f64, u64) {
        let rng = StdRng::seed_from_u64(seed);

        let (time, steps, _rng) = Self::simulate_chain(
            habitat,
            speciation_probability,
            dispersal_sampler,
            lineage_store,
            rng,
            local_partition,
        );

        (time, steps)
    }
}
