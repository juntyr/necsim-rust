#![deny(clippy::pedantic)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(stmt_expr_attributes)]
#![feature(result_flattening)]

#[macro_use]
extern crate contracts;

use std::{
    num::NonZeroU32,
    path::{Path, PathBuf},
};

use mpi::{
    environment::Universe,
    topology::{Communicator, Rank, SystemCommunicator},
    Tag,
};

use thiserror::Error;

use necsim_impls_no_std::{
    partitioning::{monolithic::live::LiveMonolithicLocalPartition, Partitioning},
    reporter::ReporterContext,
};

use necsim_impls_std::{
    event_log::recorder::EventLogRecorder,
    partitioning::monolithic::recorded::RecordedMonolithicLocalPartition,
};

mod partition;

pub use partition::{MpiLocalPartition, MpiParallelPartition, MpiRootPartition};

#[derive(Error, Debug)]
pub enum MpiPartitioningError {
    #[error("MPI has already been initialised.")]
    AlreadyInitialised,
    #[error("MPI needs a valid event log path.")]
    MissingEventLog,
    #[error(transparent)]
    InvalidEventLog(#[from] anyhow::Error),
}

pub struct MpiPartitioning {
    universe: Universe,
    world: SystemCommunicator,
    recorder: Option<EventLogRecorder>,
}

impl MpiPartitioning {
    const MPI_MIGRATION_TAG: Tag = 1;
    const MPI_PROGRESS_TAG: Tag = 0;
    const ROOT_RANK: Rank = 0;

    /// # Errors
    /// Returns `AlreadyInitialised` if MPI was already initialised.
    ///
    /// Returns `MissingEventLog` if the partitioning is non-monolithic and
    /// `event_log_path` is `None`.
    pub fn initialise(event_log_path: Option<&Path>) -> Result<Self, MpiPartitioningError> {
        mpi::initialize()
            .map(|universe| {
                let world = universe.world();

                let recorder = match event_log_path {
                    None if world.size() > 1 => Err(MpiPartitioningError::MissingEventLog),
                    None => Ok(None),
                    Some(event_log_path) => {
                        let mut event_log_path = PathBuf::from(event_log_path);
                        event_log_path.push(world.rank().to_string());

                        match EventLogRecorder::try_new(&event_log_path) {
                            Ok(recorder) => Ok(Some(recorder)),
                            Err(err) => Err(MpiPartitioningError::InvalidEventLog(err)),
                        }
                    },
                }?;

                Ok(Self {
                    universe,
                    world,
                    recorder,
                })
            })
            .ok_or(MpiPartitioningError::AlreadyInitialised)
            .flatten()
    }

    pub fn update_message_buffer_size(&mut self, size: usize) {
        if !self.is_monolithic() {
            self.universe.set_buffer_size(size)
        }
    }
}

#[contract_trait]
impl Partitioning for MpiPartitioning {
    type LocalPartition<P: ReporterContext> = MpiLocalPartition<P>;

    fn is_monolithic(&self) -> bool {
        self.world.size() <= 1
    }

    fn is_root(&self) -> bool {
        self.world.rank() == MpiPartitioning::ROOT_RANK
    }

    fn get_number_of_partitions(&self) -> NonZeroU32 {
        #[allow(clippy::cast_sign_loss)]
        NonZeroU32::new(self.world.size() as u32).unwrap()
    }

    fn into_local_partition<P: ReporterContext>(
        self,
        reporter_context: P,
    ) -> Self::LocalPartition<P> {
        #[allow(clippy::option_if_let_else)]
        if let Some(recorder) = self.recorder {
            if self.world.size() <= 1 {
                // recorded && is_monolithic
                MpiLocalPartition::RecordedMonolithic(Box::new(
                    RecordedMonolithicLocalPartition::from_reporter_and_recorder(
                        reporter_context.build_guarded(),
                        recorder,
                    ),
                ))
            } else if self.world.rank() == MpiPartitioning::ROOT_RANK {
                // recorded && !is_monolithic && is_root
                MpiLocalPartition::Root(Box::new(MpiRootPartition::new(
                    self.universe,
                    self.world,
                    reporter_context.build_guarded(),
                    recorder,
                )))
            } else {
                // recorded && !is_monolithic && !is_root
                MpiLocalPartition::Parallel(Box::new(MpiParallelPartition::new(
                    self.universe,
                    self.world,
                    recorder,
                )))
            }
        } else {
            // !recorded
            MpiLocalPartition::LiveMonolithic(Box::new(
                LiveMonolithicLocalPartition::from_reporter(reporter_context.build_guarded()),
            ))
        }
    }
}
