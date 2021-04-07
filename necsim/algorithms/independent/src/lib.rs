#![deny(clippy::pedantic)]
#![feature(never_type)]
#![allow(incomplete_features)]
#![feature(stmt_expr_attributes)]
#![feature(drain_filter)]

#[macro_use]
extern crate contracts;

use std::num::{NonZeroU64, NonZeroUsize};

use serde::Deserialize;
use serde_state::DeserializeState;

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

use necsim_impls_std::bounded::{Partition, PositiveF64};

mod almost_infinite;
mod in_memory;
mod non_spatial;

mod partitioned;

mod reporter;
use reporter::PartitionReporterProxy;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbsoluteDedupCache {
    capacity: NonZeroUsize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelativeDedupCache {
    factor: PositiveF64,
}

#[derive(Debug, Deserialize)]
pub enum DedupCache {
    Absolute(AbsoluteDedupCache),
    Relative(RelativeDedupCache),
    None,
}

#[derive(Debug, Deserialize)]
pub enum PartitionMode {
    Monolithic,
    Individuals,
    #[serde(alias = "Isolated")]
    IsolatedIndividuals(Partition),
    Landscape,
    Probabilistic,
}

#[derive(Debug)]
pub struct IndependentArguments {
    delta_t: PositiveF64,
    step_slice: NonZeroU64,
    dedup_cache: DedupCache,
    partition_mode: PartitionMode,
}

impl<'de> DeserializeState<'de, Partition> for IndependentArguments {
    fn deserialize_state<D>(seed: &mut Partition, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        use serde::de::Error;

        let raw = IndependentArgumentsRaw::deserialize(deserializer)?;

        let partition_mode = match raw.partition_mode {
            Some(partition_mode) => match partition_mode {
                PartitionMode::Monolithic | PartitionMode::IsolatedIndividuals(_)
                    if seed.partitions().get() > 1 =>
                {
                    return Err(D::Error::custom(format!(
                        "partition_mode {:?} is incompatible with non-monolithic partitioning.",
                        partition_mode
                    )))
                },
                PartitionMode::Individuals
                | PartitionMode::Landscape
                | PartitionMode::Probabilistic
                    if seed.partitions().get() == 1 =>
                {
                    return Err(D::Error::custom(format!(
                        "partition_mode {:?} is incompatible with monolithic partitioning.",
                        partition_mode
                    )))
                },
                partition_mode => partition_mode,
            },
            None => {
                if seed.partitions().get() > 1 {
                    PartitionMode::Individuals
                } else {
                    PartitionMode::Monolithic
                }
            },
        };

        Ok(IndependentArguments {
            delta_t: raw.delta_t,
            step_slice: raw.step_slice,
            dedup_cache: raw.dedup_cache,
            partition_mode,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct IndependentArgumentsRaw {
    delta_t: PositiveF64,
    step_slice: NonZeroU64,
    dedup_cache: DedupCache,
    partition_mode: Option<PartitionMode>,
}

impl Default for IndependentArgumentsRaw {
    fn default() -> Self {
        Self {
            delta_t: PositiveF64::new(1.0_f64).unwrap(),
            step_slice: NonZeroU64::new(10_u64).unwrap(),
            dedup_cache: DedupCache::Relative(RelativeDedupCache {
                factor: PositiveF64::new(2.0_f64).unwrap(),
            }),
            partition_mode: None,
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
        auxiliary: IndependentArguments,
    ) -> (f64, u64) {
        let mut proxy = PartitionReporterProxy::from(local_partition);

        let rng = SeaHash::seed_from_u64(seed);
        let lineage_store = IndependentLineageStore::default();

        let min_spec_samples: LruCache<SpeciationSample> =
            LruCache::with_capacity(match auxiliary.dedup_cache {
                DedupCache::Absolute(AbsoluteDedupCache { capacity }) => capacity.get(),
                DedupCache::Relative(RelativeDedupCache { factor }) =>
                #[allow(
                    clippy::cast_precision_loss,
                    clippy::cast_sign_loss,
                    clippy::cast_possible_truncation
                )]
                {
                    ((lineages.len() as f64) * factor.get()) as usize
                }
                DedupCache::None => 0_usize,
            });

        let (time, steps) = match auxiliary.partition_mode {
            PartitionMode::Monolithic | PartitionMode::IsolatedIndividuals(..) => {
                partitioned::monolithic::simulate(
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

        proxy.local_partition().report_progress_sync(0_u64);

        (time, steps)
    }
}
