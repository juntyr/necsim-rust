#![deny(clippy::pedantic)]
#![feature(never_type)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]

#[macro_use]
extern crate contracts;

use necsim_core::{
    cogs::{
        CoherentLineageStore, DispersalSampler, Habitat, LineageReference, RngCore,
        SpeciationProbability,
    },
    simulation::Simulation,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::classical::ClassicalActiveLineageSampler,
    coalescence_sampler::unconditional::UnconditionalCoalescenceSampler,
    emigration_exit::never::NeverEmigrationExit,
    event_sampler::unconditional::UnconditionalEventSampler,
    immigration_entry::never::NeverImmigrationEntry,
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
    fn simulate<
        H: Habitat,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, StdRng>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
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
        let coalescence_sampler = UnconditionalCoalescenceSampler::default();
        let emigration_exit = NeverEmigrationExit::default();
        let event_sampler = UnconditionalEventSampler::default();
        let immigration_entry = NeverImmigrationEntry::default();
        let active_lineage_sampler = ClassicalActiveLineageSampler::new(&lineage_store);

        let simulation = Simulation::builder()
            .habitat(habitat)
            .rng(rng)
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
}
