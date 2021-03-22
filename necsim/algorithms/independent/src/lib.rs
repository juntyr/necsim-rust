#![deny(clippy::pedantic)]
#![feature(never_type)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(stmt_expr_attributes)]

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate serde_derive_state;

use std::num::{NonZeroU64, NonZeroUsize};

use serde::Deserialize;

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
pub struct AbsoluteDedupCache {
    capacity: NonZeroUsize,
}

#[derive(Debug, Deserialize)]
pub struct RelativeDedupCache {
    factor: PositiveF64,
}

#[derive(Debug, Deserialize)]
pub enum DedupCache {
    Absolute(AbsoluteDedupCache),
    Relative(RelativeDedupCache),
    None,
}

#[derive(Debug, DeserializeState)]
#[serde(deserialize_state = "Partition")]
pub enum PartitionMode {
    Individuals,
    #[serde(alias = "Isolated")]
    IsolatedIndividuals(
        #[serde(deserialize_state_with = "deserialize_isolated_partition")] Partition,
    ),
    Landscape,
    Probabilistic,
}

fn deserialize_isolated_partition<'de, D>(
    partition: &mut Partition,
    deserializer: D,
) -> Result<Partition, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    use serde::de::Error;

    let isolated_partition = Partition::deserialize(deserializer)?;

    if isolated_partition.partitions().get() > 1 && partition.partitions().get() > 1 {
        Err(D::Error::custom(
            "IsolatedIndividuals is incompatible with non-monolithic partitioning.",
        ))
    } else {
        Ok(isolated_partition)
    }
}

#[derive(Debug, DeserializeState)]
#[serde(default)]
#[serde(deserialize_state = "Partition")]
pub struct IndependentArguments {
    delta_t: PositiveF64,
    step_slice: NonZeroU64,
    dedup_cache: DedupCache,
    #[serde(deserialize_state)]
    partition_mode: PartitionMode,
}

impl Default for IndependentArguments {
    fn default() -> Self {
        Self {
            delta_t: PositiveF64::new(1.0_f64).unwrap(),
            step_slice: NonZeroU64::new(10_u64).unwrap(),
            dedup_cache: DedupCache::Relative(RelativeDedupCache {
                factor: PositiveF64::new(2.0_f64).unwrap(),
            }),
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
        auxiliary: IndependentArguments,
    ) -> (f64, u64) {
        // TODO: how do I maintain event order during a monolithic run when events are
        //       immediately reported?

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
