use core::num::NonZeroU32;

use necsim_core::{lineage::MigratingLineage, reporter::Reporter};

use crate::reporter::ReporterContext;

pub mod iterator;
pub mod monolithic;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Partitioning: Sized {
    type LocalPartition<P: ReporterContext>: LocalPartition<P>;

    fn is_monolithic(&self) -> bool;

    #[debug_ensures(
        self.is_monolithic() -> ret,
        "monolithic partition is always root"
    )]
    fn is_root(&self) -> bool;

    #[debug_ensures(
        self.is_monolithic() == (ret.get() == 1),
        "there is only one monolithic partition"
    )]
    fn get_number_of_partitions(&self) -> NonZeroU32;

    fn into_local_partition<P: ReporterContext>(
        self,
        reporter_context: P,
    ) -> Self::LocalPartition<P>;
}

#[derive(Copy, Clone)]
pub enum MigrationMode {
    Force,
    Default,
    Hold,
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait LocalPartition<P: ReporterContext>: Sized {
    type Reporter: Reporter;
    type ImmigrantIterator<'a>: Iterator<Item = MigratingLineage>;

    fn get_reporter(&mut self) -> &mut Self::Reporter;

    fn is_root(&self) -> bool;

    #[debug_ensures(
        ret < self.get_number_of_partitions().get(),
        "partition rank is in range [0, self.get_number_of_partitions())"
    )]
    fn get_partition_rank(&self) -> u32;

    fn get_number_of_partitions(&self) -> NonZeroU32;

    fn migrate_individuals<E: Iterator<Item = (u32, MigratingLineage)>>(
        &mut self,
        emigrants: &mut E,
        emigration_mode: MigrationMode,
        immigration_mode: MigrationMode,
    ) -> Self::ImmigrantIterator<'_>;

    fn reduce_vote_continue(&self, local_continue: bool) -> bool;

    fn reduce_vote_min_time(&self, local_time: f64) -> Result<f64, f64>;

    fn wait_for_termination(&mut self) -> bool;

    fn reduce_global_time_steps(&self, local_time: f64, local_steps: u64) -> (f64, u64);
}
