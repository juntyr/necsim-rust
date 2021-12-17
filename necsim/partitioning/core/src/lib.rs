#![deny(clippy::pedantic)]
#![no_std]
#![feature(generic_associated_types)]

extern crate alloc;

#[macro_use]
extern crate contracts;

use necsim_core::{
    lineage::MigratingLineage,
    reporter::{boolean::Boolean, Reporter},
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

pub mod context;
pub mod iterator;
pub mod partition;

use context::ReporterContext;
use partition::Partition;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Partitioning: Sized {
    type LocalPartition<R: Reporter>: LocalPartition<R>;
    type Auxiliary;

    fn is_monolithic(&self) -> bool;

    #[debug_ensures(
        self.is_monolithic() -> ret,
        "monolithic partition is always root"
    )]
    fn is_root(&self) -> bool;

    fn get_partition(&self) -> Partition;

    fn into_local_partition<R: Reporter, P: ReporterContext<Reporter = R>>(
        self,
        reporter_context: P,
        auxiliary: Self::Auxiliary,
    ) -> anyhow::Result<Self::LocalPartition<R>>;
}

#[derive(Copy, Clone)]
pub enum MigrationMode {
    Force,
    Default,
    Hold,
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait LocalPartition<R: Reporter>: Sized {
    type Reporter: Reporter;
    type IsLive: Boolean;
    type ImmigrantIterator<'a>: Iterator<Item = MigratingLineage>
    where
        Self: 'a;

    fn get_reporter(&mut self) -> &mut Self::Reporter;

    fn is_root(&self) -> bool;

    fn get_partition(&self) -> Partition;

    fn migrate_individuals<E: Iterator<Item = (u32, MigratingLineage)>>(
        &mut self,
        emigrants: &mut E,
        emigration_mode: MigrationMode,
        immigration_mode: MigrationMode,
    ) -> Self::ImmigrantIterator<'_>;

    fn reduce_vote_continue(&self, local_continue: bool) -> bool;

    fn reduce_vote_min_time(&self, local_time: PositiveF64) -> Result<PositiveF64, PositiveF64>;

    fn wait_for_termination(&mut self) -> bool;

    fn reduce_global_time_steps(
        &self,
        local_time: NonNegativeF64,
        local_steps: u64,
    ) -> (NonNegativeF64, u64);

    fn report_progress_sync(&mut self, remaining: u64);

    fn finalise_reporting(self);
}
