use alloc::vec::Vec;
use core::num::NonZeroU32;

use necsim_core::{lineage::MigratingLineage, reporter::Reporter};

use crate::reporter::{GuardedReporter, ReporterContext};

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

pub struct MonolithicLocalPartition<P: ReporterContext> {
    reporter: GuardedReporter<P::Reporter, P::Finaliser>,
    loopback: Vec<MigratingLineage>,
}

#[contract_trait]
impl<P: ReporterContext> LocalPartition<P> for MonolithicLocalPartition<P> {
    type ImmigrantIterator<'a> = ImmigrantPopIterator<'a>;
    type Reporter = P::Reporter;

    fn get_reporter(&mut self) -> &mut Self::Reporter {
        &mut self.reporter
    }

    fn is_root(&self) -> bool {
        true
    }

    fn get_partition_rank(&self) -> u32 {
        0
    }

    fn get_number_of_partitions(&self) -> NonZeroU32 {
        unsafe { NonZeroU32::new_unchecked(1) }
    }

    fn migrate_individuals<E: Iterator<Item = (u32, MigratingLineage)>>(
        &mut self,
        emigrants: &mut E,
        _emigration_mode: MigrationMode,
        _immigration_mode: MigrationMode,
    ) -> Self::ImmigrantIterator<'_> {
        for (_, emigrant) in emigrants {
            self.loopback.push(emigrant);
        }

        ImmigrantPopIterator::new(&mut self.loopback)
    }

    fn reduce_vote_continue(&self, local_continue: bool) -> bool {
        local_continue
    }

    fn reduce_vote_min_time(&self, local_time: f64) -> Result<f64, f64> {
        Ok(local_time)
    }

    fn wait_for_termination(&mut self) -> bool {
        !self.loopback.is_empty()
    }

    fn reduce_global_time_steps(&self, local_time: f64, local_steps: u64) -> (f64, u64) {
        (local_time, local_steps)
    }
}

pub struct ImmigrantPopIterator<'i> {
    immigrants: &'i mut Vec<MigratingLineage>,
}

impl<'i> ImmigrantPopIterator<'i> {
    pub fn new(immigrants: &'i mut Vec<MigratingLineage>) -> Self {
        Self { immigrants }
    }
}

impl<'i> Iterator for ImmigrantPopIterator<'i> {
    type Item = MigratingLineage;

    fn next(&mut self) -> Option<Self::Item> {
        self.immigrants.pop()
    }
}

impl<P: ReporterContext> MonolithicLocalPartition<P> {
    pub fn from_reporter(reporter_guard: GuardedReporter<P::Reporter, P::Finaliser>) -> Self {
        Self {
            reporter: reporter_guard,
            loopback: Vec::new(),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct MonolithicPartitioning(());

impl Default for MonolithicPartitioning {
    fn default() -> Self {
        Self(())
    }
}

#[contract_trait]
impl Partitioning for MonolithicPartitioning {
    type LocalPartition<P: ReporterContext> = MonolithicLocalPartition<P>;

    fn is_monolithic(&self) -> bool {
        true
    }

    fn is_root(&self) -> bool {
        true
    }

    fn get_number_of_partitions(&self) -> NonZeroU32 {
        unsafe { NonZeroU32::new_unchecked(1) }
    }

    fn into_local_partition<P: ReporterContext>(
        self,
        reporter_context: P,
    ) -> Self::LocalPartition<P> {
        MonolithicLocalPartition::from_reporter(reporter_context.build_guarded())
    }
}
