#![deny(clippy::pedantic)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(stmt_expr_attributes)]

#[macro_use]
extern crate contracts;

use std::num::NonZeroU32;

use mpi::{
    environment::Universe,
    topology::{Communicator, Rank, SystemCommunicator},
    Tag,
};

use thiserror::Error;

use necsim_core::reporter::Reporter;

use necsim_impls_std::event_log::recorder::EventLogRecorder;
use necsim_partitioning_core::{context::ReporterContext, Partitioning};
use necsim_partitioning_monolithic::{
    live::LiveMonolithicLocalPartition, recorded::RecordedMonolithicLocalPartition,
};

mod partition;

pub use partition::{MpiLocalPartition, MpiParallelPartition, MpiRootPartition};

#[derive(Error, Debug)]
pub enum MpiPartitioningError {
    #[error("MPI has already been initialised.")]
    AlreadyInitialised,
}

#[derive(Error, Debug)]
pub enum MpiLocalPartitionError {
    #[error("MPI needs a valid event log path.")]
    MissingEventLog,
}

pub struct MpiPartitioning {
    universe: Universe,
    world: SystemCommunicator,
}

impl MpiPartitioning {
    const MPI_MIGRATION_TAG: Tag = 1;
    const MPI_PROGRESS_TAG: Tag = 0;
    const ROOT_RANK: Rank = 0;

    /// # Errors
    ///
    /// Returns `AlreadyInitialised` if MPI was already initialised.
    pub fn initialise() -> Result<Self, MpiPartitioningError> {
        mpi::initialize()
            .map(|universe| Self {
                world: universe.world(),
                universe,
            })
            .ok_or(MpiPartitioningError::AlreadyInitialised)
    }

    pub fn update_message_buffer_size(&mut self, size: usize) {
        if !self.is_monolithic() {
            self.universe.set_buffer_size(size)
        }
    }
}

#[contract_trait]
impl Partitioning for MpiPartitioning {
    type Auxiliary = Option<EventLogRecorder>;
    type LocalPartition<R: Reporter> = MpiLocalPartition<R>;

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

    fn get_rank(&self) -> u32 {
        #[allow(clippy::cast_sign_loss)]
        (self.world.rank() as u32)
    }

    /// # Errors
    ///
    /// Returns `MissingEventLog` if the local partition is non-monolithic and
    /// the `auxiliary` event log is `None`.
    fn into_local_partition<R: Reporter, P: ReporterContext<Reporter = R>>(
        self,
        reporter_context: P,
        auxiliary: Self::Auxiliary,
    ) -> anyhow::Result<Self::LocalPartition<R>> {
        #[allow(clippy::option_if_let_else)]
        if let Some(event_log) = auxiliary {
            Ok(if self.world.size() <= 1 {
                // recorded && is_monolithic
                MpiLocalPartition::RecordedMonolithic(Box::new(
                    RecordedMonolithicLocalPartition::try_from_context_and_recorder(
                        reporter_context,
                        event_log,
                    )?,
                ))
            } else if self.world.rank() == MpiPartitioning::ROOT_RANK {
                // recorded && !is_monolithic && is_root
                MpiLocalPartition::Root(Box::new(MpiRootPartition::new(
                    self.universe,
                    self.world,
                    reporter_context.try_build()?,
                    event_log,
                )))
            } else {
                // recorded && !is_monolithic && !is_root
                MpiLocalPartition::Parallel(Box::new(MpiParallelPartition::new(
                    self.universe,
                    self.world,
                    event_log,
                )))
            })
        } else if self.world.size() <= 1 {
            // !recorded && monolithic
            Ok(MpiLocalPartition::LiveMonolithic(Box::new(
                LiveMonolithicLocalPartition::try_from_context(reporter_context)?,
            )))
        } else {
            Err(anyhow::Error::new(MpiLocalPartitionError::MissingEventLog))
        }
    }
}
