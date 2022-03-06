use std::{
    fmt,
    marker::PhantomData,
    num::NonZeroU32,
    time::{Duration, Instant},
};

use mpi::{
    collective::{CommunicatorCollectives, Root, SystemOperation},
    datatype::Equivalence,
    environment::Universe,
    point_to_point::{Destination, Source},
    request::{CancelGuard, LocalScope, Request},
    topology::{Communicator, SystemCommunicator},
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
use necsim_partitioning_core::{
    iterator::ImmigrantPopIterator, partition::Partition, LocalPartition, MigrationMode,
};

use crate::{
    partition::utils::{reduce_lexicographic_min_time_rank, MpiMigratingLineage},
    MpiPartitioning,
};

pub struct MpiParallelPartition<'p, R: Reporter> {
    _universe: Universe,
    world: SystemCommunicator,
    scope: &'p LocalScope<'p>,
    mpi_local_continue: &'p mut bool,
    mpi_global_continue: &'p mut bool,
    mpi_local_remaining: &'p mut u64,
    mpi_migration_buffers: &'p mut [Vec<MigratingLineage>],
    last_report_time: Instant,
    progress: Option<Request<'p, &'p LocalScope<'p>>>,
    migration_buffers: Box<[Vec<MigratingLineage>]>,
    last_migration_times: Box<[Instant]>,
    emigration_requests: Box<[Option<Request<'p, &'p LocalScope<'p>>>]>,
    barrier: Option<Request<'p, &'p LocalScope<'p>>>,
    communicated_since_last_barrier: bool,
    recorder: EventLogRecorder,
    _marker: PhantomData<(&'p (), R)>,
}

impl<'p, R: Reporter> fmt::Debug for MpiParallelPartition<'p, R> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(MpiParallelPartition)).finish()
    }
}

impl<'p, R: Reporter> Drop for MpiParallelPartition<'p, R> {
    fn drop(&mut self) {
        if let Some(progress) = self.progress.take() {
            std::mem::drop(CancelGuard::from(progress));
        }

        for request in self.emigration_requests.iter_mut() {
            if let Some(request) = request.take() {
                std::mem::drop(CancelGuard::from(request));
            }
        }

        if let Some(barrier) = self.barrier.take() {
            std::mem::drop(CancelGuard::from(barrier));
        }
    }
}

impl<'p, R: Reporter> MpiParallelPartition<'p, R> {
    const MPI_MIGRATION_WAIT_TIME: Duration = Duration::from_millis(100_u64);
    const MPI_PROGRESS_WAIT_TIME: Duration = Duration::from_millis(100_u64);

    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        universe: Universe,
        world: SystemCommunicator,
        scope: &'p LocalScope<'p>,
        mpi_local_continue: &'p mut bool,
        mpi_global_continue: &'p mut bool,
        mpi_local_remaining: &'p mut u64,
        mpi_migration_buffers: &'p mut [Vec<MigratingLineage>],
        mut recorder: EventLogRecorder,
    ) -> Self {
        recorder.set_event_filter(R::ReportSpeciation::VALUE, R::ReportDispersal::VALUE);

        #[allow(clippy::cast_sign_loss)]
        let world_size = world.size() as usize;

        let mut migration_buffers = Vec::with_capacity(world_size);
        migration_buffers.resize_with(world_size, Vec::new);

        let mut emigration_requests = Vec::with_capacity(world_size);
        emigration_requests.resize_with(world_size, || None);

        let now = Instant::now();

        Self {
            _universe: universe,
            world,
            scope,
            mpi_local_continue,
            mpi_global_continue,
            mpi_local_remaining,
            mpi_migration_buffers,
            last_report_time: now.checked_sub(Self::MPI_PROGRESS_WAIT_TIME).unwrap_or(now),
            progress: None,
            migration_buffers: migration_buffers.into_boxed_slice(),
            last_migration_times: vec![now; world_size].into_boxed_slice(),
            emigration_requests: emigration_requests.into_boxed_slice(),
            barrier: None,
            communicated_since_last_barrier: false,
            recorder,
            _marker: PhantomData,
        }
    }
}

#[contract_trait]
impl<'p, R: Reporter> LocalPartition<'p, R> for MpiParallelPartition<'p, R> {
    type ImmigrantIterator<'a>
    where
        'p: 'a,
        R: 'a,
    = ImmigrantPopIterator<'a>;
    type IsLive = False;
    type Reporter = Self;

    fn get_reporter(&mut self) -> &mut Self::Reporter {
        self
    }

    fn is_root(&self) -> bool {
        false
    }

    fn get_partition(&self) -> Partition {
        #[allow(clippy::cast_sign_loss)]
        let rank = self.world.rank() as u32;
        #[allow(clippy::cast_sign_loss)]
        let size = unsafe { NonZeroU32::new_unchecked(self.world.size() as u32) };

        unsafe { Partition::new_unchecked(rank, size) }
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
            self.migration_buffers[partition as usize].push(emigrant);
        }

        let self_rank_index = self.get_partition().rank() as usize;

        let now = Instant::now();

        // Receive incomming immigrating lineages
        if match immigration_mode {
            MigrationMode::Force => true,
            MigrationMode::Default => {
                now.duration_since(self.last_migration_times[self_rank_index])
                    >= Self::MPI_MIGRATION_WAIT_TIME
            },
            MigrationMode::Hold => false,
        } {
            self.last_migration_times[self_rank_index] = now;

            let immigration_buffer = &mut self.migration_buffers[self_rank_index];

            let any_process = self.world.any_process();

            // Probe MPI to receive
            while let Some((msg, status)) =
                any_process.immediate_matched_probe_with_tag(MpiPartitioning::MPI_MIGRATION_TAG)
            {
                #[allow(clippy::cast_sign_loss)]
                let number_immigrants =
                    status.count(MpiMigratingLineage::equivalent_datatype()) as usize;

                let receive_start = immigration_buffer.len();

                #[allow(clippy::uninit_vec)]
                // Safety: The uninitialised `number_immigrants` items are initialised in the
                //         following `matched_receive_into` call
                unsafe {
                    immigration_buffer.reserve(number_immigrants);
                    immigration_buffer.set_len(receive_start + number_immigrants);

                    let immigration_slice = MpiMigratingLineage::from_mut_slice(
                        &mut immigration_buffer[receive_start..],
                    );

                    msg.matched_receive_into(immigration_slice);
                }
            }
        }

        // Send outgoing emigrating lineages
        for rank in 0..self.get_partition().size().get() {
            let rank_index = rank as usize;

            if rank_index != self_rank_index
                && match emigration_mode {
                    MigrationMode::Force => true,
                    MigrationMode::Default => {
                        now.duration_since(self.last_migration_times[rank_index])
                            >= Self::MPI_PROGRESS_WAIT_TIME
                    },
                    MigrationMode::Hold => false,
                }
            {
                // Check if the prior send request has finished
                if let Some(request) = self.emigration_requests[rank_index].take() {
                    if let Err(request) = request.test() {
                        self.emigration_requests[rank_index] = Some(request);
                    } else {
                        self.mpi_migration_buffers[rank_index].clear();
                    }
                }

                let emigration_buffer = &mut self.migration_buffers[rank_index];

                // Send a new non-empty request iff the prior one has finished
                if self.emigration_requests[rank_index].is_none() && !emigration_buffer.is_empty() {
                    self.last_migration_times[rank_index] = now;

                    // MPI cannot terminate in this round since this partition gave up work
                    self.communicated_since_last_barrier = true;

                    // Safety: emigration requests barrier protects from mutability conflicts
                    let local_emigration_buffer =
                        unsafe { &mut *(&mut self.mpi_migration_buffers[rank_index] as *mut _) };

                    std::mem::swap(emigration_buffer, local_emigration_buffer);

                    let local_emigration_slice =
                        MpiMigratingLineage::from_slice(local_emigration_buffer);

                    #[allow(clippy::cast_possible_wrap)]
                    let receiver_process = self.world.process_at_rank(rank as i32);

                    self.emigration_requests[rank_index] =
                        Some(receiver_process.immediate_synchronous_send_with_tag(
                            self.scope,
                            local_emigration_slice,
                            MpiPartitioning::MPI_MIGRATION_TAG,
                        ));
                }
            }
        }

        ImmigrantPopIterator::new(&mut self.migration_buffers[self.get_partition().rank() as usize])
    }

    fn reduce_vote_continue(&self, local_continue: bool) -> bool {
        let mut global_continue = local_continue;

        self.world.all_reduce_into(
            &local_continue,
            &mut global_continue,
            SystemOperation::logical_or(),
        );

        global_continue
    }

    fn reduce_vote_min_time(&self, local_time: PositiveF64) -> Result<PositiveF64, PositiveF64> {
        let local_partition_rank = self.get_partition().rank();

        let (global_min_time, global_min_rank) =
            reduce_lexicographic_min_time_rank(self.world, local_time, local_partition_rank);

        if global_min_rank == local_partition_rank {
            Ok(local_time)
        } else {
            Err(global_min_time)
        }
    }

    fn wait_for_termination(&mut self) -> bool {
        // This partition can only terminate once all migrations have been processed
        for buffer in self.migration_buffers.iter() {
            if !buffer.is_empty() {
                return true;
            }
        }
        for request in self.emigration_requests.iter() {
            if request.is_some() {
                return true;
            }
        }
        for buffer in self.mpi_migration_buffers.iter() {
            if !buffer.is_empty() {
                return true;
            }
        }

        // Create a new termination attempt if the last one failed
        let barrier = self.barrier.take().unwrap_or_else(|| {
            // Safety: the barrier protects from mutability conflicts
            let local_continue: &'p mut bool = unsafe { &mut *(self.mpi_local_continue as *mut _) };
            let global_continue: &'p mut bool =
                unsafe { &mut *(self.mpi_global_continue as *mut _) };

            *local_continue = self.communicated_since_last_barrier;
            self.communicated_since_last_barrier = false;
            *global_continue = false;

            self.world.immediate_all_reduce_into(
                self.scope,
                local_continue,
                global_continue,
                SystemOperation::logical_or(),
            )
        });

        match barrier.test() {
            Ok(_) => *self.mpi_global_continue,
            Err(barrier) => {
                self.barrier = Some(barrier);

                true
            },
        }
    }

    fn reduce_global_time_steps(
        &self,
        local_time: NonNegativeF64,
        local_steps: u64,
    ) -> (NonNegativeF64, u64) {
        let mut global_time_max = 0.0_f64;
        let mut global_steps_sum = 0_u64;

        self.world.all_reduce_into(
            &local_time.get(),
            &mut global_time_max,
            SystemOperation::max(),
        );
        self.world
            .all_reduce_into(&local_steps, &mut global_steps_sum, SystemOperation::sum());

        // Safety: `global_time_max` is the max of multiple `NonNegativeF64`s
        //         communicated through MPI
        let global_time_max = unsafe { NonNegativeF64::new_unchecked(global_time_max) };

        (global_time_max, global_steps_sum)
    }

    fn report_progress_sync(&mut self, remaining: u64) {
        let root_process = self.world.process_at_rank(MpiPartitioning::ROOT_RANK);

        root_process.gather_into(&remaining);
    }

    fn finalise_reporting(self) {
        std::mem::drop(self);
    }
}

impl<'p, R: Reporter> Reporter for MpiParallelPartition<'p, R> {
    impl_report!(speciation(&mut self, speciation: MaybeUsed<R::ReportSpeciation>) {
        self.recorder.record_speciation(speciation);
    });

    impl_report!(dispersal(&mut self, dispersal: MaybeUsed<R::ReportDispersal>) {
        self.recorder.record_dispersal(dispersal);
    });

    impl_report!(progress(&mut self, remaining: MaybeUsed<R::ReportProgress>) {
        if *self.mpi_local_remaining == *remaining {
            return;
        }

        if *remaining > 0 || self.barrier.is_none() {
            let now = Instant::now();

            if now.duration_since(self.last_report_time) >= Self::MPI_PROGRESS_WAIT_TIME {
                let progress = self.progress.take().unwrap_or_else(|| {
                    // Safety: the progress barrier protects from mutability conflicts
                    let local_remaining: &'p mut u64 = unsafe { &mut *(self.mpi_local_remaining as *mut _) };

                    self.last_report_time = now;
                    *local_remaining = *remaining;

                    let root_process = self.world.process_at_rank(MpiPartitioning::ROOT_RANK);

                    root_process.immediate_send_with_tag(
                        self.scope,
                        local_remaining,
                        MpiPartitioning::MPI_PROGRESS_TAG,
                    )
                });

                if let Err(progress) = progress.test() {
                    self.progress = Some(progress);
                }
            }
        }
    });
}
