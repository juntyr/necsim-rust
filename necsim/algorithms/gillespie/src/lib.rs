#![deny(clippy::pedantic)]
#![feature(never_type)]

#[macro_use]
extern crate contracts;

use std::marker::PhantomData;

use necsim_core::{
    cogs::{CoherentLineageStore, DispersalSampler, Habitat, LineageReference, RngCore},
    simulation::{partial::event_sampler::PartialSimulation, Simulation},
};

use necsim_impls_no_std::{
    cogs::{
        coalescence_sampler::unconditional::UnconditionalCoalescenceSampler,
        emigration_exit::never::NeverEmigrationExit,
        event_sampler::gillespie::unconditional::UnconditionalGillespieEventSampler,
        immigration_entry::never::NeverImmigrationEntry,
        speciation_probability::uniform::UniformSpeciationProbability,
    },
    partitioning::LocalPartition,
    reporter::ReporterContext,
};

use necsim_impls_std::cogs::{
    active_lineage_sampler::gillespie::GillespieActiveLineageSampler, rng::std::StdRng,
};

mod almost_infinite;
mod in_memory;
mod non_spatial;

pub struct GillespieSimulation;

impl GillespieSimulation {
    /// Simulates the Gillespie coalescence algorithm on the `habitat` with
    /// `dispersal` and lineages from `lineage_store`.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=RxC` where `habitat` has dimension
    /// `RxC`.
    fn simulate<
        H: Habitat,
        D: DispersalSampler<H, StdRng>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        P: ReporterContext,
        L: LocalPartition<P>,
    >(
        habitat_in: H,
        dispersal_sampler_in: D,
        lineage_store_in: S,
        speciation_probability_per_generation: f64,
        seed: u64,
        local_partition: &mut L,
    ) -> (f64, u64) {
        let mut rng = StdRng::seed_from_u64(seed);
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation);
        let emigration_exit = NeverEmigrationExit::default();
        let coalescence_sampler = UnconditionalCoalescenceSampler::default();
        let event_sampler = UnconditionalGillespieEventSampler::default();

        // Pack a PartialSimulation to initialise the GillespieActiveLineageSampler
        let partial_simulation = PartialSimulation {
            habitat: habitat_in,
            speciation_probability,
            dispersal_sampler: dispersal_sampler_in,
            lineage_reference: PhantomData::<R>,
            lineage_store: lineage_store_in,
            emigration_exit,
            coalescence_sampler,
            rng: PhantomData::<StdRng>,
        };

        let active_lineage_sampler =
            GillespieActiveLineageSampler::new(&partial_simulation, &event_sampler, &mut rng);

        // Unpack the PartialSimulation to create the full Simulation
        let PartialSimulation {
            habitat,
            speciation_probability,
            dispersal_sampler,
            lineage_reference,
            lineage_store,
            emigration_exit,
            coalescence_sampler,
            rng: _,
        } = partial_simulation;

        let immigration_entry = NeverImmigrationEntry::default();

        let simulation = Simulation::builder()
            .habitat(habitat)
            .rng(rng)
            .speciation_probability(speciation_probability)
            .dispersal_sampler(dispersal_sampler)
            .lineage_reference(lineage_reference)
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
