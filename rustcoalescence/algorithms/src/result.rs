use std::{error::Error as StdError, fmt};

use necsim_core::{cogs::RngCore, lineage::Lineage};
use necsim_core_bond::NonNegativeF64;

use necsim_impls_no_std::cogs::active_lineage_sampler::resuming::lineage::ExceptionalLineage;

pub enum SimulationOutcome<G: RngCore> {
    Done {
        time: NonNegativeF64,
        steps: u64,
    },
    Paused {
        time: NonNegativeF64,
        steps: u64,
        lineages: Vec<Lineage>,
        rng: G,
    },
}

#[derive(Debug)]
pub enum ResumeError<E: StdError + Send + Sync + 'static> {
    Sample(Vec<ExceptionalLineage>),
    Simulate(E),
}

impl<E: StdError + Send + Sync + 'static> std::error::Error for ResumeError<E> {}

impl<E: StdError + Send + Sync + 'static> From<E> for ResumeError<E> {
    fn from(err: E) -> Self {
        Self::Simulate(err)
    }
}

impl<E: StdError + Send + Sync + 'static> fmt::Display for ResumeError<E> {
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
