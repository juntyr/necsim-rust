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
};

use thiserror::Error;

use necsim_impls_no_std::{
    partitioning::{MonolithicLocalPartition, Partitioning},
    reporter::ReporterContext,
};

mod partition;

pub use partition::{MpiLocalPartition, MpiParallelPartition, MpiRootPartition};

#[derive(Error, Debug)]
#[error("MPI has already been initialised.")]
pub struct MpiAlreadyInitialisedError;

pub struct MpiPartitioning {
    universe: Universe,
    world: SystemCommunicator,
}

impl MpiPartitioning {
    const ROOT_RANK: Rank = 0;

    /// # Errors
    /// Returns `MpiAlreadyInitialisedError` if MPI was already initialised.
    pub fn initialise() -> Result<Self, MpiAlreadyInitialisedError> {
        mpi::initialize()
            .map(|universe| Self {
                world: universe.world(),
                universe,
            })
            .ok_or(MpiAlreadyInitialisedError)
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
        if self.is_monolithic() {
            MpiLocalPartition::Monolithic(MonolithicLocalPartition::from_reporter(
                reporter_context.build_guarded(),
            ))
        } else if self.is_root() {
            MpiLocalPartition::Root(MpiRootPartition::from_universe_world_and_reporter(
                self.universe,
                self.world,
                reporter_context.build_guarded(),
            ))
        } else {
            MpiLocalPartition::Parallel(MpiParallelPartition::from_universe_and_world(
                self.universe,
                self.world,
            ))
        }
    }
}
