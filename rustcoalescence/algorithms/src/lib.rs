#![deny(clippy::pedantic)]
#![feature(generic_associated_types)]

use std::{error::Error as StdError, fmt, marker::PhantomData};

use serde::{Deserialize, Serialize};

use necsim_core::{
    cogs::{LineageReference, LineageStore, MathsCore, RngCore},
    lineage::Lineage,
    reporter::Reporter,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::resuming::lineage::ExceptionalLineage,
    origin_sampler::pre_sampler::OriginPreSampler,
};
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_scenarios::Scenario;

pub trait AlgorithmParamters {
    type Arguments;
    type Error: StdError + Send + Sync + 'static;
}

pub trait Algorithm<O: Scenario<Self::MathsCore, Self::Rng>, R: Reporter, P: LocalPartition<R>>:
    Sized + AlgorithmParamters
{
    type MathsCore: MathsCore;
    type Rng: RngCore<Self::MathsCore>;
    type LineageReference: LineageReference<Self::MathsCore, O::Habitat>;
    type LineageStore: LineageStore<Self::MathsCore, O::Habitat, Self::LineageReference>;

    /// # Errors
    ///
    /// Returns a `Self::Error` if initialising the fresh simulation or running
    ///  the algorithm failed
    fn initialise_and_simulate<I: Iterator<Item = u64>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<Self::MathsCore, I>,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<AlgorithmResult<Self::MathsCore, Self::Rng>, Self::Error>;

    /// # Errors
    ///
    /// Returns a `ContinueError<Self::Error>` if initialising the resuming
    ///  simulation or running the algorithm failed
    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    fn resume_and_simulate<I: Iterator<Item = u64>, L: ExactSizeIterator<Item = Lineage>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<Self::MathsCore, I>,
        lineages: L,
        resume_after: Option<NonNegativeF64>,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<AlgorithmResult<Self::MathsCore, Self::Rng>, ContinueError<Self::Error>>;

    /// # Errors
    ///
    /// Returns a `ContinueError<Self::Error>` if fixing up the restarting
    ///  simulation (incl. running the algorithm) failed
    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    fn fixup_for_restart<I: Iterator<Item = u64>, L: ExactSizeIterator<Item = Lineage>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<Self::MathsCore, I>,
        lineages: L,
        restart_at: PositiveF64,
        _fixup_strategy: RestartFixUpStrategy,
        local_partition: &mut P,
    ) -> Result<AlgorithmResult<Self::MathsCore, Self::Rng>, ContinueError<Self::Error>>;
}

pub enum AlgorithmResult<M: MathsCore, G: RngCore<M>> {
    Done {
        time: NonNegativeF64,
        steps: u64,
    },
    Paused {
        time: NonNegativeF64,
        steps: u64,
        lineages: Vec<Lineage>,
        rng: G,
        marker: PhantomData<M>,
    },
}

#[derive(Debug)]
pub enum ContinueError<E: StdError + Send + Sync + 'static> {
    Sample(Vec<ExceptionalLineage>),
    Simulate(E),
}

impl<E: StdError + Send + Sync + 'static> fmt::Display for ContinueError<E> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Sample(exceptional_lineages) => {
                writeln!(
                    fmt,
                    "{} lineage(s) are incompatible with the scenario, e.g.",
                    exceptional_lineages.len()
                )?;

                if let Some((child, parent)) = exceptional_lineages.iter().find_map(|e| match e {
                    ExceptionalLineage::Coalescence { child, parent } => Some((child, parent)),
                    _ => None,
                }) {
                    writeln!(
                        fmt,
                        "- Lineage #{} at ({}, {}, {}) is at the same indexed location as Lineage \
                         #{}",
                        child.global_reference,
                        child.indexed_location.location().x(),
                        child.indexed_location.location().y(),
                        child.indexed_location.index(),
                        parent,
                    )?;
                }

                if let Some(lineage) = exceptional_lineages.iter().find_map(|e| match e {
                    ExceptionalLineage::OutOfDeme(lineage) => Some(lineage),
                    _ => None,
                }) {
                    writeln!(
                        fmt,
                        "- Lineage #{} at ({}, {}, {}) is outside the deme at its location",
                        lineage.global_reference,
                        lineage.indexed_location.location().x(),
                        lineage.indexed_location.location().y(),
                        lineage.indexed_location.index(),
                    )?;
                }

                if let Some(lineage) = exceptional_lineages.iter().find_map(|e| match e {
                    ExceptionalLineage::OutOfHabitat(lineage) => Some(lineage),
                    _ => None,
                }) {
                    writeln!(
                        fmt,
                        "- Lineage #{} at ({}, {}, {}) is outside the habitable area",
                        lineage.global_reference,
                        lineage.indexed_location.location().x(),
                        lineage.indexed_location.location().y(),
                        lineage.indexed_location.index(),
                    )?;
                }

                Ok(())
            },
            Self::Simulate(err) => fmt::Display::fmt(err, fmt),
        }
    }
}

impl<E: StdError + Send + Sync + 'static> std::error::Error for ContinueError<E> {}

impl<E: StdError + Send + Sync + 'static> From<E> for ContinueError<E> {
    fn from(err: E) -> Self {
        Self::Simulate(err)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct RestartFixUpStrategy {
    #[serde(alias = "deme", alias = "ood")]
    pub out_of_deme: OutOfDemeStrategy,
    #[serde(alias = "habitat", alias = "ooh")]
    pub out_of_habitat: OutOfHabitatStrategy,
    #[serde(alias = "dup", alias = "coa")]
    pub coalescence: CoalescenceStrategy,
}

impl Default for RestartFixUpStrategy {
    fn default() -> Self {
        Self {
            out_of_deme: OutOfDemeStrategy::Abort,
            out_of_habitat: OutOfHabitatStrategy::Abort,
            coalescence: CoalescenceStrategy::Abort,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OutOfDemeStrategy {
    Abort,
    Dispersal,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OutOfHabitatStrategy {
    Abort,
    #[serde(alias = "Uniform")]
    UniformDispersal,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CoalescenceStrategy {
    Abort,
    Coalescence,
}
