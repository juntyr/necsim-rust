#![deny(clippy::pedantic)]
#![feature(generic_associated_types)]

use std::error::Error as StdError;

use necsim_core::{
    cogs::{LineageReference, LineageStore, MathsCore, RngCore},
    lineage::Lineage,
    reporter::Reporter,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_impls_no_std::cogs::origin_sampler::pre_sampler::OriginPreSampler;
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_scenarios::Scenario;

pub mod result;
pub mod strategy;

use result::{ResumeError, SimulationOutcome};
use strategy::RestartFixUpStrategy;

pub trait AlgorithmParamters {
    type Arguments;
    type Error: StdError + Send + Sync + 'static;
}

pub trait AlgorithmDefaults {
    type MathsCore: MathsCore;
}

pub trait Algorithm<
    'p,
    M: MathsCore,
    O: Scenario<M, Self::Rng>,
    R: Reporter,
    P: LocalPartition<'p, R>,
>: Sized + AlgorithmParamters + AlgorithmDefaults
{
    type Rng: RngCore<M>;
    type LineageReference: LineageReference<M, O::Habitat>;
    type LineageStore: LineageStore<M, O::Habitat, Self::LineageReference>;

    /// # Errors
    ///
    /// Returns a `Self::Error` if initialising the fresh simulation or running
    ///  the algorithm failed
    fn initialise_and_simulate<I: Iterator<Item = u64>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<M, I>,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<SimulationOutcome<M, Self::Rng>, Self::Error>;

    /// # Errors
    ///
    /// Returns a `ContinueError<Self::Error>` if initialising the resuming
    ///  simulation or running the algorithm failed
    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    fn resume_and_simulate<I: Iterator<Item = u64>, L: ExactSizeIterator<Item = Lineage>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<M, I>,
        lineages: L,
        resume_after: Option<NonNegativeF64>,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<SimulationOutcome<M, Self::Rng>, ResumeError<Self::Error>>;

    /// # Errors
    ///
    /// Returns a `ContinueError<Self::Error>` if fixing up the restarting
    ///  simulation (incl. running the algorithm) failed
    #[allow(clippy::type_complexity, clippy::too_many_arguments)]
    fn fixup_for_restart<I: Iterator<Item = u64>, L: ExactSizeIterator<Item = Lineage>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<M, I>,
        lineages: L,
        restart_at: PositiveF64,
        fixup_strategy: RestartFixUpStrategy,
        local_partition: &mut P,
    ) -> Result<SimulationOutcome<M, Self::Rng>, ResumeError<Self::Error>>;
}
