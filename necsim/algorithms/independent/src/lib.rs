#![deny(clippy::pedantic)]
#![feature(never_type)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(stmt_expr_attributes)]

#[macro_use]
extern crate contracts;

use std::{
    convert::TryFrom,
    num::{NonZeroU32, NonZeroU64, NonZeroUsize},
};

use anyhow::Result;
use serde::Deserialize;
use thiserror::Error;

use necsim_core::{
    cogs::{DispersalSampler, Habitat, RngCore, SpeciationProbability, SpeciationSample},
    lineage::Lineage,
};

use necsim_impls_no_std::{
    cache::DirectMappedCache as LruCache,
    cogs::{
        emigration_exit::independent::choice::{
            always::AlwaysEmigrationChoice, probabilistic::ProbabilisticEmigrationChoice,
        },
        lineage_store::independent::IndependentLineageStore,
        rng::seahash::SeaHash,
    },
    decomposition::Decomposition,
    partitioning::LocalPartition,
    reporter::ReporterContext,
};

use necsim_impls_std::bounded::PositiveF64;

mod almost_infinite;
mod in_memory;
mod non_spatial;

mod partitioned;

mod reporter;
use reporter::PartitionReporterProxy;

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum DedupCache {
    Absolute(NonZeroUsize),
    Relative(PositiveF64),
    None,
}

#[derive(Debug, Error)]
#[error("{0} is not in range [0, {1}].")]
#[allow(clippy::module_name_repetitions)]
pub struct PartitionRankOutOfBounds(u32, NonZeroU32);

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(try_from = "IsolatedPartitionRaw")]
pub struct IsolatedPartition {
    rank: u32,
    partitions: NonZeroU32,
}

impl TryFrom<IsolatedPartitionRaw> for IsolatedPartition {
    type Error = PartitionRankOutOfBounds;

    fn try_from(raw: IsolatedPartitionRaw) -> Result<Self, Self::Error> {
        if raw.rank < raw.partitions.get() {
            Ok(Self {
                rank: raw.rank,
                partitions: raw.partitions,
            })
        } else {
            Err(PartitionRankOutOfBounds(raw.rank, raw.partitions))
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
struct IsolatedPartitionRaw {
    rank: u32,
    partitions: NonZeroU32,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum PartitionMode {
    Individuals,
    #[serde(alias = "Isolated")]
    IsolatedIndividuals(IsolatedPartition),
    Landscape,
    Probabilistic,
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(default)]
pub struct IndependentArguments {
    delta_t: PositiveF64,
    step_slice: NonZeroU64,
    dedup_cache: DedupCache,
    partition_mode: PartitionMode,
}

impl Default for IndependentArguments {
    fn default() -> Self {
        Self {
            delta_t: PositiveF64::new(1.0_f64).unwrap(),
            step_slice: NonZeroU64::new(10_u64).unwrap(),
            dedup_cache: DedupCache::Relative(PositiveF64::new(2.0_f64).unwrap()),
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
    ) -> (f64, u64) {
        // TODO: how do I maintain event order during a monolithic run when events are
        //       immediately reported?

        let mut proxy = PartitionReporterProxy::from(local_partition);

        let rng = SeaHash::seed_from_u64(seed);
        let lineage_store = IndependentLineageStore::default();

        let min_spec_samples: LruCache<SpeciationSample> =
            LruCache::with_capacity(match auxiliary.dedup_cache {
                DedupCache::Absolute(capacity) => capacity.get(),
                DedupCache::Relative(scalar) =>
                #[allow(
                    clippy::cast_precision_loss,
                    clippy::cast_sign_loss,
                    clippy::cast_possible_truncation
                )]
                {
                    ((lineages.len() as f64) * scalar.get()) as usize
                }
                DedupCache::None => 0_usize,
            });

        let (time, steps) = match auxiliary.partition_mode {
            PartitionMode::Individuals | PartitionMode::IsolatedIndividuals(..) => {
                partitioned::individuals::simulate(
                    habitat,
                    rng,
                    speciation_probability,
                    dispersal_sampler,
                    lineage_store,
                    lineages.into(),
                    &mut proxy,
                    min_spec_samples,
                    auxiliary,
                )
            },
            PartitionMode::Landscape => partitioned::landscape::simulate(
                habitat,
                rng,
                speciation_probability,
                dispersal_sampler,
                lineage_store,
                lineages.into(),
                &mut proxy,
                decomposition,
                AlwaysEmigrationChoice::default(),
                min_spec_samples,
                auxiliary,
            ),
            PartitionMode::Probabilistic => partitioned::landscape::simulate(
                habitat,
                rng,
                speciation_probability,
                dispersal_sampler,
                lineage_store,
                lineages.into(),
                &mut proxy,
                decomposition,
                ProbabilisticEmigrationChoice::default(),
                min_spec_samples,
                auxiliary,
            ),
        };

        proxy.report_total_progress(0_u64);

        (time, steps)
    }
}
