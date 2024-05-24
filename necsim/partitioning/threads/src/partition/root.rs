use std::{fmt, marker::PhantomData, ops::ControlFlow, time::Duration};

use necsim_core::{
    impl_report,
    lineage::MigratingLineage,
    reporter::{boolean::False, Reporter},
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_partitioning_core::{
    iterator::ImmigrantPopIterator, partition::Partition, LocalPartition, MigrationMode,
};

use crate::vote::Vote;

pub struct ThreadsRootPartition<'p, R: Reporter> {
    partition: Partition,
    vote_any: Vote<bool>,
    vote_min_time: Vote<(PositiveF64, u32)>,
    vote_time_steps: Vote<(NonNegativeF64, u64)>,
    _migration_interval: Duration,
    _progress_interval: Duration,
    _marker: PhantomData<(&'p (), R)>,
}

impl<'p, R: Reporter> fmt::Debug for ThreadsRootPartition<'p, R> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(ThreadsRootPartition)).finish()
    }
}

impl<'p, R: Reporter> ThreadsRootPartition<'p, R> {
    #[must_use]
    pub(crate) fn new(
        partition: Partition,
        vote_any: &Vote<bool>,
        vote_min_time: &Vote<(PositiveF64, u32)>,
        vote_time_steps: &Vote<(NonNegativeF64, u64)>,
        migration_interval: Duration,
        progress_interval: Duration,
    ) -> Self {
        Self {
            partition,
            vote_any: vote_any.clone(),
            vote_min_time: vote_min_time.clone(),
            vote_time_steps: vote_time_steps.clone(),
            _migration_interval: migration_interval,
            _progress_interval: progress_interval,
            _marker: PhantomData::<(&'p (), R)>,
        }
    }
}

#[contract_trait]
impl<'p, R: Reporter> LocalPartition<'p, R> for ThreadsRootPartition<'p, R> {
    type ImmigrantIterator<'a> = ImmigrantPopIterator<'a> where 'p: 'a, R: 'a;
    type IsLive = False;
    type Reporter = Self;

    fn get_reporter(&mut self) -> &mut Self::Reporter {
        self
    }

    fn is_root(&self) -> bool {
        true
    }

    fn get_partition(&self) -> Partition {
        self.partition
    }

    fn migrate_individuals<'a, E: Iterator<Item = (u32, MigratingLineage)>>(
        &'a mut self,
        _emigrants: &mut E,
        _emigration_mode: MigrationMode,
        _immigration_mode: MigrationMode,
    ) -> Self::ImmigrantIterator<'a>
    where
        'p: 'a,
    {
        unimplemented!()
    }

    fn reduce_vote_any(&mut self, vote: bool) -> bool {
        self.vote_any.vote(|acc| match acc {
            None => vote,
            Some(acc) => *acc || vote,
        })
    }

    fn reduce_vote_min_time(
        &mut self,
        local_time: PositiveF64,
    ) -> Result<PositiveF64, PositiveF64> {
        let vote = (local_time, self.partition.rank());

        let result = self.vote_min_time.vote(|acc| match acc {
            None => vote,
            Some(acc) => vote.min(*acc),
        });

        if result.1 == self.partition.rank() {
            Ok(result.0)
        } else {
            Err(result.0)
        }
    }

    fn wait_for_termination(&mut self) -> ControlFlow<(), ()> {
        unimplemented!()
    }

    fn reduce_global_time_steps(
        &mut self,
        local_time: NonNegativeF64,
        local_steps: u64,
    ) -> (NonNegativeF64, u64) {
        self.vote_time_steps.vote(|acc| match acc {
            None => (local_time, local_steps),
            Some((global_time, global_steps)) => {
                (local_time.max(*global_time), local_steps + (*global_steps))
            },
        })
    }

    fn report_progress_sync(&mut self, _remaining: u64) {
        unimplemented!()
    }

    fn finalise_reporting(self) {
        unimplemented!()
    }
}

impl<'p, R: Reporter> Reporter for ThreadsRootPartition<'p, R> {
    impl_report!(speciation(&mut self, _speciation: MaybeUsed<R::ReportSpeciation>) {
        unimplemented!()
    });

    impl_report!(dispersal(&mut self, _dispersal: MaybeUsed<R::ReportDispersal>) {
        unimplemented!()
    });

    impl_report!(progress(&mut self, _remaining: MaybeUsed<R::ReportProgress>) {
        unimplemented!()
    });
}
