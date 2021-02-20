#![deny(clippy::pedantic)]
#![feature(never_type)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(stmt_expr_attributes)]

#[macro_use]
extern crate contracts;

use std::collections::VecDeque;

use anyhow::Result;
use lru_set::LruSet;

use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, Habitat, MinSpeciationTrackingEventSampler,
        RngCore, SingularActiveLineageSampler, SpeciationProbability, SpeciationSample,
    },
    lineage::{GlobalLineageReference, Lineage},
    simulation::Simulation,
};

use necsim_impls_no_std::{
    cogs::{
        active_lineage_sampler::independent::{
            event_time_sampler::exp::ExpEventTimeSampler, IndependentActiveLineageSampler,
        },
        coalescence_sampler::independent::IndependentCoalescenceSampler,
        emigration_exit::never::NeverEmigrationExit,
        event_sampler::independent::IndependentEventSampler,
        immigration_entry::never::NeverImmigrationEntry,
        lineage_store::independent::IndependentLineageStore,
        rng::seahash::SeaHash,
    },
    partitioning::LocalPartition,
    reporter::ReporterContext,
};

mod almost_infinite;
mod in_memory;
mod non_spatial;

mod reporter;
use reporter::DeduplicatingReporterProxy;

#[derive(Copy, Clone, Debug)]
pub enum DedupMode {
    Static(usize),
    Dynamic(f64),
    None,
}

#[derive(Copy, Clone, Debug)]
pub struct IndependentArguments {
    pub delta_t: f64,
    pub step_slice: usize,
    pub dedup_mode: DedupMode,
}

impl Default for IndependentArguments {
    fn default() -> Self {
        Self {
            delta_t: 1.0_f64,
            step_slice: 10_usize,
            dedup_mode: DedupMode::Dynamic(2.0_f64),
        }
    }
}

pub struct IndependentSimulation;

impl IndependentSimulation {
    /// Simulates the independent coalescence algorithm on the `habitat` with
    /// `dispersal` and lineages from `lineage_store`.
    fn simulate<
        H: Habitat,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, SeaHash>,
        R: ReporterContext,
        P: LocalPartition<R>,
    >(
        habitat: H,
        speciation_probability: N,
        dispersal_sampler: D,
        lineages: Vec<Lineage>,
        seed: u64,
        local_partition: &mut P,
        auxiliary: &IndependentArguments,
    ) -> Result<(f64, u64)> {
        anyhow::ensure!(
            auxiliary.delta_t > 0.0_f64,
            "Independent algorithm delta_t={} must be positive.",
            auxiliary.delta_t
        );
        anyhow::ensure!(
            auxiliary.step_slice > 0,
            "Independent algorithm step_slice={} must be positive.",
            auxiliary.step_slice
        );
        anyhow::ensure!(
            if let DedupMode::Dynamic(scalar) = auxiliary.dedup_mode {
                scalar >= 0.0_f64
            } else {
                true
            },
            "Independent algorithm dedup_mode={:?} dynamic scalar must be non-negative.",
            auxiliary.dedup_mode,
        );

        let step_slice = auxiliary.step_slice as u64;

        let mut proxy = DeduplicatingReporterProxy::from(local_partition);

        let rng = SeaHash::seed_from_u64(seed);
        let lineage_store = IndependentLineageStore::default();
        let emigration_exit = NeverEmigrationExit::default();
        let coalescence_sampler = IndependentCoalescenceSampler::default();
        let event_sampler = IndependentEventSampler::default();
        let immigration_entry = NeverImmigrationEntry::default();
        let active_lineage_sampler =
            IndependentActiveLineageSampler::empty(ExpEventTimeSampler::new(auxiliary.delta_t));

        let mut simulation = Simulation::builder()
            .habitat(habitat)
            .rng(rng)
            .speciation_probability(speciation_probability)
            .dispersal_sampler(dispersal_sampler)
            .lineage_reference(std::marker::PhantomData::<GlobalLineageReference>)
            .lineage_store(lineage_store)
            .emigration_exit(emigration_exit)
            .coalescence_sampler(coalescence_sampler)
            .event_sampler(event_sampler)
            .immigration_entry(immigration_entry)
            .active_lineage_sampler(active_lineage_sampler)
            .build();

        let mut min_spec_samples: LruSet<SpeciationSample> =
            LruSet::with_capacity(match auxiliary.dedup_mode {
                DedupMode::Static(capacity) => capacity,
                DedupMode::Dynamic(scalar) =>
                #[allow(
                    clippy::cast_precision_loss,
                    clippy::cast_sign_loss,
                    clippy::cast_possible_truncation
                )]
                {
                    ((lineages.len() as f64) * scalar) as usize
                }
                DedupMode::None => 0_usize,
            });

        let mut total_steps = 0_u64;
        let mut max_time = 0.0_f64;

        let mut lineages: VecDeque<Lineage> = lineages.into();

        while !lineages.is_empty()
            || simulation.active_lineage_sampler().number_active_lineages() > 0
            || proxy.local_partition().wait_for_termination()
        {
            proxy.report_total_progress(
                (lineages.len() + simulation.active_lineage_sampler().number_active_lineages())
                    as u64,
            );

            let previous_task = simulation
                .active_lineage_sampler_mut()
                .replace_active_lineage(lineages.pop_front());

            let previous_speciation_sample =
                simulation.event_sampler_mut().replace_min_speciation(None);

            if let Some(previous_speciation_sample) = previous_speciation_sample {
                if min_spec_samples.insert(previous_speciation_sample) {
                    if let Some(previous_task) = previous_task {
                        if previous_task.is_active() {
                            lineages.push_back(previous_task);
                        }
                    }
                }
            }

            let (new_time, new_steps) = simulation.simulate_incremental(step_slice, &mut proxy);

            total_steps += new_steps;
            max_time = max_time.max(new_time);
        }

        proxy.report_total_progress(0_u64);

        Ok((max_time, total_steps))
    }
}
