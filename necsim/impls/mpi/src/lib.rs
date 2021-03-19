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
    partitioning::{monolithic::MonolithicLocalPartition, Partitioning},
    reporter::ReporterContext,
};

mod partition;

pub use partition::{MpiLocalPartition, MpiParallelPartition, MpiRootPartition};

#[derive(Error, Debug)]
pub enum MpiPartitioningError {
    #[error("MPI has already been initialised.")]
    AlreadyInitialised,
    #[error("MPI needs a valid event log path.")]
    MissingEventLog,
}

pub struct MpiPartitioning {
    universe: Universe,
    world: SystemCommunicator,
    event_log_path: Option<PathBuf>,
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

                let event_log_path = match event_log_path {
                    None if world.size() > 1 => Err(MpiPartitioningError::MissingEventLog),
                    _ => Ok(event_log_path.map(PathBuf::from)),
                }?;

                Ok(Self {
                    universe,
                    world,
                    event_log_path,
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
        if let Some(event_log_path) = &self.event_log_path {
            // !self.is_monolithic()
            if self.is_root() {
                MpiLocalPartition::Root(Box::new(MpiRootPartition::new(
                    self.universe,
                    self.world,
                    reporter_context.build_guarded(),
                    &event_log_path,
                )))
            } else {
                MpiLocalPartition::Parallel(Box::new(MpiParallelPartition::new(
                    self.universe,
                    self.world,
                    &event_log_path,
                )))
            }
        } else {
            // self.is_monolithic()
            MpiLocalPartition::Monolithic(Box::new(MonolithicLocalPartition::from_reporter(
                reporter_context.build_guarded(),
            )))
        }
    }
}
