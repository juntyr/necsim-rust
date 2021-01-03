#![deny(clippy::pedantic)]
#![feature(never_type)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]

#[macro_use]
extern crate contracts;

use hashbrown::{hash_map::RawEntryMut, HashMap, HashSet};

use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, Habitat, HabitatToU64Injection,
        IncoherentLineageStore, LineageReference, MinSpeciationTrackingEventSampler, RngCore,
        SingularActiveLineageSampler, SpeciationProbability, SpeciationSample,
    },
    event::Event,
    reporter::{EventFilter, Reporter},
    simulation::Simulation,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::independent::{
        event_time_sampler::exp::ExpEventTimeSampler, IndependentActiveLineageSampler,
    },
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    event_sampler::independent::IndependentEventSampler,
    rng::fixedseahash::FixedSeaHash,
};

use necsim_impls_no_std::reporter::ReporterContext;

mod almost_infinite;
mod in_memory;
mod non_spatial;

pub struct IndependentSimulation;

struct DeduplicatingReporterProxy<'r, H: Habitat, R: LineageReference<H>, P: Reporter<H, R>> {
    reporter: &'r mut P,
    event_deduplicator: HashMap<Event<H, R>, ()>,
}

impl<'r, H: Habitat, R: LineageReference<H>, P: Reporter<H, R>> EventFilter
    for DeduplicatingReporterProxy<'r, H, R, P>
{
    const REPORT_DISPERSAL: bool = P::REPORT_DISPERSAL;
    const REPORT_SPECIATION: bool = P::REPORT_SPECIATION;
}

impl<'r, H: Habitat, R: LineageReference<H>, P: Reporter<H, R>> Reporter<H, R>
    for DeduplicatingReporterProxy<'r, H, R, P>
{
    fn report_event(&mut self, event: &Event<H, R>) {
        if let RawEntryMut::Vacant(entry) = self.event_deduplicator.raw_entry_mut().from_key(event)
        {
            self.reporter
                .report_event(entry.insert(event.clone(), ()).0)
        }
    }
}

impl<'r, H: Habitat, R: LineageReference<H>, P: Reporter<H, R>>
    DeduplicatingReporterProxy<'r, H, R, P>
{
    fn from(reporter: &'r mut P) -> Self {
        Self {
            reporter,
            event_deduplicator: HashMap::new(),
        }
    }
}

impl IndependentSimulation {
    /// Simulates the independent coalescence algorithm on the `habitat` with
    /// `dispersal` and lineages from `lineage_store`.
    fn simulate<
        H: HabitatToU64Injection,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, FixedSeaHash>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
        P: ReporterContext,
    >(
        habitat: H,
        speciation_probability: N,
        dispersal_sampler: D,
        lineage_store: S,
        seed: u64,
        reporter_context: P,
    ) -> (f64, u64) {
        const SIMULATION_STEP_SLICE: u64 = 10_u64;

        reporter_context.with_reporter(|reporter| {
            let mut reporter = DeduplicatingReporterProxy::from(reporter);

            let rng = FixedSeaHash::seed_from_u64(seed);
            let coalescence_sampler = IndependentCoalescenceSampler::default();
            let event_sampler = IndependentEventSampler::default();
            let active_lineage_sampler =
                IndependentActiveLineageSampler::empty(ExpEventTimeSampler::new(1.0_f64));

            let mut simulation = Simulation::builder()
                .habitat(habitat)
                .rng(rng)
                .speciation_probability(speciation_probability)
                .dispersal_sampler(dispersal_sampler)
                .lineage_reference(std::marker::PhantomData::<R>)
                .lineage_store(lineage_store)
                .coalescence_sampler(coalescence_sampler)
                .event_sampler(event_sampler)
                .active_lineage_sampler(active_lineage_sampler)
                .build();

            let mut lineages: Vec<R> = simulation
                .lineage_store()
                .iter_local_lineage_references()
                .collect();

            let mut min_spec_samples: HashSet<SpeciationSample> = HashSet::new();

            let mut total_steps = 0_u64;
            let mut max_time = 0.0_f64;

            while let Some(active_lineage) = lineages.pop() {
                let prev_task = simulation.with_mut_split_active_lineage_sampler_and_rng(
                    |active_lineage_sampler, simulation, _rng| {
                        active_lineage_sampler.replace_active_lineage(
                            Some(active_lineage),
                            &mut simulation.lineage_store,
                        )
                    },
                );

                while simulation.active_lineage_sampler().number_active_lineages() > 0 {
                    let old_min_spec_sample =
                        simulation.event_sampler_mut().replace_min_speciation(None);

                    let (new_time, new_steps) =
                        simulation.simulate_incremental(SIMULATION_STEP_SLICE, &mut reporter);

                    let new_min_spec_sample = simulation
                        .event_sampler_mut()
                        .replace_min_speciation(old_min_spec_sample);

                    total_steps += new_steps;
                    max_time = max_time.max(new_time);

                    if let Some(new_min_spec_sample) = new_min_spec_sample {
                        if !min_spec_samples.insert(new_min_spec_sample) {
                            break;
                        }
                    }
                }

                simulation.with_mut_split_active_lineage_sampler_and_rng(
                    |active_lineage_sampler, simulation, _rng| {
                        active_lineage_sampler
                            .replace_active_lineage(prev_task, &mut simulation.lineage_store)
                    },
                );
            }

            (max_time, total_steps)
        })
    }
}
