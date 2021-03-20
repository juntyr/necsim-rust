use alloc::vec::Vec;
use core::num::NonZeroU32;

use necsim_core::lineage::MigratingLineage;

use crate::{
    partitioning::{iterator::ImmigrantPopIterator, LocalPartition, MigrationMode, Partitioning},
    reporter::{GuardedReporter, ReporterContext},
};

#[allow(clippy::module_name_repetitions)]
pub struct LiveMonolithicPartitioning(());

impl Default for LiveMonolithicPartitioning {
    fn default() -> Self {
        Self(())
    }
}

#[contract_trait]
impl Partitioning for LiveMonolithicPartitioning {
    type Auxiliary = ();
    type Error = !;
    type LocalPartition<P: ReporterContext> = LiveMonolithicLocalPartition<P>;

    fn is_monolithic(&self) -> bool {
        true
    }

    fn is_root(&self) -> bool {
        true
    }

    fn get_number_of_partitions(&self) -> NonZeroU32 {
        unsafe { NonZeroU32::new_unchecked(1) }
    }

    fn get_rank(&self) -> u32 {
        0
    }

    fn into_local_partition<P: ReporterContext>(
        self,
        reporter_context: P,
        _auxiliary: Self::Auxiliary,
    ) -> Result<Self::LocalPartition<P>, Self::Error> {
        Ok(LiveMonolithicLocalPartition::from_reporter(
            reporter_context.build_guarded(),
        ))
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct LiveMonolithicLocalPartition<P: ReporterContext> {
    reporter: GuardedReporter<P::Reporter, P::Finaliser>,
    loopback: Vec<MigratingLineage>,
}

#[contract_trait]
impl<P: ReporterContext> LocalPartition<P> for LiveMonolithicLocalPartition<P> {
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

impl<P: ReporterContext> LiveMonolithicLocalPartition<P> {
    pub fn from_reporter(reporter_guard: GuardedReporter<P::Reporter, P::Finaliser>) -> Self {
        Self {
            reporter: reporter_guard,
            loopback: Vec::new(),
        }
    }
}