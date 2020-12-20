#![deny(clippy::pedantic)]
#![feature(never_type)]

#[macro_use]
extern crate contracts;

use std::marker::PhantomData;

use necsim_core::{
    cogs::{CoherentLineageStore, Habitat, LineageReference, RngCore, SeparableDispersalSampler},
    simulation::{partial::event_sampler::PartialSimulation, Simulation},
};

use necsim_impls_no_std::cogs::{
    coalescence_sampler::conditional::ConditionalCoalescenceSampler,
    event_sampler::gillespie::conditional::ConditionalGillespieEventSampler,
};
use necsim_impls_std::cogs::{
    active_lineage_sampler::gillespie::GillespieActiveLineageSampler, rng::std::StdRng,
};

use necsim_impls_no_std::reporter::ReporterContext;

mod almost_infinite;
mod in_memory;
mod non_spatial;

pub struct SkippingGillespieSimulation;

impl SkippingGillespieSimulation {
    /// Simulates the Gillespie coalescence algorithm with self-dispersal event
    /// skipping on the `habitat` with `dispersal` and lineages from
    /// `lineage_store`.
    fn simulate<
        H: Habitat,
        D: SeparableDispersalSampler<H, StdRng>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        P: ReporterContext,
    >(
        habitat: H,
        dispersal_sampler: D,
        lineage_store: S,
        speciation_probability_per_generation: f64,
        seed: u64,
        reporter_context: P,
    ) -> (f64, u64) {
        reporter_context.with_reporter(|reporter| {
            let mut rng = StdRng::seed_from_u64(seed);
            let coalescence_sampler = ConditionalCoalescenceSampler::default();
            let event_sampler = ConditionalGillespieEventSampler::default();

            // Pack a PartialSimulation to initialise the GillespieActiveLineageSampler
            let partial_simulation = PartialSimulation {
                speciation_probability_per_generation,
                habitat,
                dispersal_sampler,
                lineage_reference: PhantomData::<R>,
                lineage_store,
                coalescence_sampler,
                rng: PhantomData::<StdRng>,
            };

            let active_lineage_sampler =
                GillespieActiveLineageSampler::new(&partial_simulation, &event_sampler, &mut rng);

            // Unpack the PartialSimulation to create the full Simulation
            let PartialSimulation {
                speciation_probability_per_generation,
                habitat,
                dispersal_sampler,
                lineage_reference,
                lineage_store,
                coalescence_sampler,
                rng: _,
            } = partial_simulation;

            let simulation = Simulation::builder()
                .speciation_probability_per_generation(speciation_probability_per_generation)
                .habitat(habitat)
                .rng(rng)
                .dispersal_sampler(dispersal_sampler)
                .lineage_reference(lineage_reference)
                .lineage_store(lineage_store)
                .coalescence_sampler(coalescence_sampler)
                .event_sampler(event_sampler)
                .active_lineage_sampler(active_lineage_sampler)
                .build();

            let (time, steps) = simulation.simulate(reporter);

            (time, steps)
        })
    }
}
