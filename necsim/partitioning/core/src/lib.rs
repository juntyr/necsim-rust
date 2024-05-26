#![deny(clippy::pedantic)]
#![no_std]

extern crate alloc;

#[macro_use]
extern crate contracts;

use core::ops::ControlFlow;

use necsim_core::{
    lineage::MigratingLineage,
    reporter::{boolean::Boolean, Reporter},
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

pub mod context;
pub mod iterator;
pub mod partition;

use context::ReporterContext;
use partition::{Partition, PartitionSize};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Partitioning: Sized {
    type LocalPartition<'p, R: Reporter>: LocalPartition<'p, R>;
    type Auxiliary;

    fn get_size(&self) -> PartitionSize;

    fn with_local_partition<
        R: Reporter,
        P: ReporterContext<Reporter = R>,
        A: Data,
        Q: Data + serde::Serialize + serde::de::DeserializeOwned,
    >(
        self,
        reporter_context: P,
        auxiliary: Self::Auxiliary,
        args: A,
        inner: for<'p> fn(Self::LocalPartition<'p, R>, A) -> Q,
        fold: fn(Q, Q) -> Q,
    ) -> anyhow::Result<Q>;
}

pub trait Data: Send + Clone {}
impl<T: Send + Clone> Data for T {}

#[derive(Copy, Clone)]
pub enum MigrationMode {
    Force,
    Default,
    Hold,
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait LocalPartition<'p, R: Reporter>: Sized {
    type Reporter: Reporter;
    type IsLive: Boolean;
    type ImmigrantIterator<'a>: Iterator<Item = MigratingLineage>
    where
        Self: 'a,
        'p: 'a;

    fn get_reporter(&mut self) -> &mut Self::Reporter;

    fn is_root(&self) -> bool;

    fn get_partition(&self) -> Partition;

    fn migrate_individuals<'a, E: Iterator<Item = (u32, MigratingLineage)>>(
        &'a mut self,
        emigrants: &mut E,
        emigration_mode: MigrationMode,
        immigration_mode: MigrationMode,
    ) -> Self::ImmigrantIterator<'a>
    where
        'p: 'a;

    fn reduce_vote_any(&mut self, vote: bool) -> bool;

    fn reduce_vote_min_time(&mut self, local_time: PositiveF64)
        -> Result<PositiveF64, PositiveF64>;

    fn wait_for_termination(&mut self) -> ControlFlow<(), ()>;

    fn reduce_global_time_steps(
        &mut self,
        local_time: NonNegativeF64,
        local_steps: u64,
    ) -> (NonNegativeF64, u64);

    fn report_progress_sync(&mut self, remaining: u64);

    fn finalise_reporting(self);
}
