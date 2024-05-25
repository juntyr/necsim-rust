use std::{
    fmt,
    marker::PhantomData,
    ops::ControlFlow,
    sync::mpsc::{Receiver, SyncSender, TrySendError},
    time::{Duration, Instant},
};

use necsim_core::{
    impl_report,
    lineage::MigratingLineage,
    reporter::{
        boolean::{Boolean, False},
        Reporter,
    },
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_impls_std::event_log::recorder::EventLogRecorder;
use necsim_partitioning_core::{partition::Partition, LocalPartition, MigrationMode};

use crate::vote::Vote;

use super::ImmigrantPopIterator;

pub struct ThreadsParallelPartition<'p, R: Reporter> {
    partition: Partition,
    vote_any: Vote<bool>,
    vote_min_time: Vote<(PositiveF64, u32)>,
    vote_time_steps: Vote<(NonNegativeF64, u64)>,
    emigration_buffers: Box<[Vec<MigratingLineage>]>,
    emigration_channels: Box<[SyncSender<Vec<MigratingLineage>>]>,
    immigration_buffers: Vec<Vec<MigratingLineage>>,
    immigration_channel: Receiver<Vec<MigratingLineage>>,
    last_migration_times: Box<[Instant]>,
    communicated_since_last_barrier: bool,
    migration_interval: Duration,
    recorder: EventLogRecorder,
    _progress_interval: Duration,
    _marker: PhantomData<(&'p (), R)>,
}

impl<'p, R: Reporter> fmt::Debug for ThreadsParallelPartition<'p, R> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(ThreadsRootPartition)).finish()
    }
}

impl<'p, R: Reporter> ThreadsParallelPartition<'p, R> {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub(crate) fn new(
        partition: Partition,
        vote_any: &Vote<bool>,
        vote_min_time: &Vote<(PositiveF64, u32)>,
        vote_time_steps: &Vote<(NonNegativeF64, u64)>,
        emigration_channels: &[SyncSender<Vec<MigratingLineage>>],
        immigration_channel: Receiver<Vec<MigratingLineage>>,
        migration_interval: Duration,
        mut recorder: EventLogRecorder,
        progress_interval: Duration,
    ) -> Self {
        recorder.set_event_filter(R::ReportSpeciation::VALUE, R::ReportDispersal::VALUE);

        let partition_size = partition.size().get() as usize;

        let mut emigration_buffers = Vec::with_capacity(partition_size);
        emigration_buffers.resize_with(partition_size, Vec::new);

        let now = Instant::now();

        Self {
            partition,
            vote_any: vote_any.clone(),
            vote_min_time: vote_min_time.clone(),
            vote_time_steps: vote_time_steps.clone(),
            emigration_buffers: emigration_buffers.into_boxed_slice(),
            emigration_channels: Vec::from(emigration_channels).into_boxed_slice(),
            immigration_buffers: Vec::new(),
            immigration_channel,
            last_migration_times: vec![
                now.checked_sub(migration_interval).unwrap_or(now);
                partition_size
            ]
            .into_boxed_slice(),
            communicated_since_last_barrier: false,
            migration_interval,
            recorder,
            _progress_interval: progress_interval,
            _marker: PhantomData::<(&'p (), R)>,
        }
    }
}

#[contract_trait]
impl<'p, R: Reporter> LocalPartition<'p, R> for ThreadsParallelPartition<'p, R> {
    type ImmigrantIterator<'a> = ImmigrantPopIterator<'a> where 'p: 'a, R: 'a;
    type IsLive = False;
    type Reporter = Self;

    fn get_reporter(&mut self) -> &mut Self::Reporter {
        self
    }

    fn is_root(&self) -> bool {
        false
    }

    fn get_partition(&self) -> Partition {
        self.partition
    }

    fn migrate_individuals<'a, E: Iterator<Item = (u32, MigratingLineage)>>(
        &'a mut self,
        emigrants: &mut E,
        emigration_mode: MigrationMode,
        immigration_mode: MigrationMode,
    ) -> Self::ImmigrantIterator<'a>
    where
        'p: 'a,
    {
        for (partition, emigrant) in emigrants {
            self.emigration_buffers[partition as usize].push(emigrant);
        }

        let self_rank_index = self.get_partition().rank() as usize;

        let now = Instant::now();

        // Receive incoming immigrating lineages
        if match immigration_mode {
            MigrationMode::Force => true,
            MigrationMode::Default => {
                now.duration_since(self.last_migration_times[self_rank_index])
                    >= self.migration_interval
            },
            MigrationMode::Hold => false,
        } {
            self.last_migration_times[self_rank_index] = now;

            self.immigration_buffers
                .extend(self.immigration_channel.try_iter());
        }

        // Send outgoing emigrating lineages
        for partition in self.partition.size().partitions() {
            let rank_index = partition.rank() as usize;

            if rank_index != self_rank_index
                && match emigration_mode {
                    MigrationMode::Force => true,
                    MigrationMode::Default => {
                        now.duration_since(self.last_migration_times[rank_index])
                            >= self.migration_interval
                    },
                    MigrationMode::Hold => false,
                }
            {
                let emigration_buffer = &mut self.emigration_buffers[rank_index];

                if !emigration_buffer.is_empty() {
                    let emigration_buffer_message = std::mem::take(emigration_buffer);

                    // Send a new non-empty request iff there is capacity
                    match self.emigration_channels[rank_index].try_send(emigration_buffer_message) {
                        Ok(()) => {
                            self.last_migration_times[rank_index] = now;

                            // we cannot terminate in this round since this partition gave up work
                            self.communicated_since_last_barrier = true;
                        },
                        Err(TrySendError::Full(emigration_buffer_message)) => {
                            *emigration_buffer = emigration_buffer_message;
                        },
                        Err(TrySendError::Disconnected(_)) => {
                            panic!("threads partitioning migration channel disconnected")
                        },
                    }
                }
            }
        }

        ImmigrantPopIterator::new(&mut self.immigration_buffers)
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

impl<'p, R: Reporter> Reporter for ThreadsParallelPartition<'p, R> {
    impl_report!(speciation(&mut self, speciation: MaybeUsed<R::ReportSpeciation>) {
        self.recorder.record_speciation(speciation);
    });

    impl_report!(dispersal(&mut self, dispersal: MaybeUsed<R::ReportDispersal>) {
        self.recorder.record_dispersal(dispersal);
    });

    impl_report!(progress(&mut self, _remaining: MaybeUsed<R::ReportProgress>) {
        unimplemented!()
    });
}
