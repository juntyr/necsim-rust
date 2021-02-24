#![deny(clippy::pedantic)]
#![feature(never_type)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(stmt_expr_attributes)]

#[macro_use]
extern crate contracts;

use anyhow::Result;
use lru_set::LruSet;

use necsim_core::{
    cogs::{DispersalSampler, Habitat, RngCore, SpeciationProbability, SpeciationSample},
    lineage::Lineage,
};

use necsim_impls_no_std::{
    cogs::{lineage_store::independent::IndependentLineageStore, rng::seahash::SeaHash},
    decomposition::Decomposition,
    partitioning::LocalPartition,
    reporter::ReporterContext,
};

mod almost_infinite;
mod in_memory;
mod non_spatial;

mod partitioned;

mod reporter;
use reporter::PartitionReporterProxy;

#[derive(Copy, Clone, Debug)]
pub enum DedupMode {
    Static(usize),
    Dynamic(f64),
    None,
}

#[derive(Copy, Clone, Debug)]
pub enum PartitionMode {
    Landscape,
    Individuals,
}

#[derive(Copy, Clone, Debug)]
pub struct IndependentArguments {
    pub delta_t: f64,
    pub step_slice: usize,
    pub dedup_mode: DedupMode,
    pub partition_mode: PartitionMode,
}

impl Default for IndependentArguments {
    fn default() -> Self {
        Self {
            delta_t: 1.0_f64,
            step_slice: 10_usize,
            dedup_mode: DedupMode::Dynamic(2.0_f64),
            partition_mode: PartitionMode::Individuals,
        }
    }
}

pub struct IndependentSimulation;

impl IndependentSimulation {
    /// Simulates the independent coalescence algorithm on the `habitat` with
    /// `dispersal` and lineages from `lineage_store`.
    #[allow(clippy::too_many_arguments)]
    fn simulate<
        H: Habitat,
        C: Decomposition<H>,
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
        decomposition: C,
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

        // TODO: how do I maintain event order during a monolithic run when events are
        //       immediately reported?

        let mut proxy = PartitionReporterProxy::from(local_partition);

        let rng = SeaHash::seed_from_u64(seed);
        let lineage_store = IndependentLineageStore::default();

        let min_spec_samples: LruSet<SpeciationSample> =
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

        let (time, steps) = match auxiliary.partition_mode {
            PartitionMode::Individuals => partitioned::individuals::simulate(
                habitat,
                rng,
                speciation_probability,
                dispersal_sampler,
                lineage_store,
                lineages.into(),
                &mut proxy,
                min_spec_samples,
                auxiliary,
            ),
            PartitionMode::Landscape => partitioned::landscape::simulate(
                habitat,
                rng,
                speciation_probability,
                dispersal_sampler,
                lineage_store,
                lineages.into(),
                &mut proxy,
                decomposition,
                min_spec_samples,
                auxiliary,
            ),
        }?;

        proxy.report_total_progress(0_u64);

        Ok((time, steps))
    }
}
