#![deny(clippy::pedantic)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]

#[macro_use]
extern crate contracts;

use core::num::NonZeroU32;
use std::marker::PhantomData;

use mpi::{
    environment::Universe,
    topology::{Communicator, SystemCommunicator},
};

use thiserror::Error;

use necsim_core::{
    event::Event,
    reporter::{EventFilter, Reporter},
};

use necsim_impls_no_std::{
    partitioning::{MonolithicPartition, ParallelPartition, Partition, Partitioning},
    reporter::ReporterContext,
};

#[derive(Error, Debug)]
#[error("MPI has already been initialised.")]
pub struct MpiAlreadyInitialisedError;

pub struct MpiPartitioning {
    universe: Universe,
    world: SystemCommunicator,
}

impl MpiPartitioning {
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
    type ParallelPartition<R: Reporter> = MpiParallelPartition<R>;

    fn is_monolithic(&self) -> bool {
        self.world.size() <= 1
    }

    fn is_root(&self) -> bool {
        self.world.rank() == 0
    }

    fn get_number_of_partitions(&self) -> NonZeroU32 {
        #[allow(clippy::cast_sign_loss)]
        NonZeroU32::new(self.world.size() as u32).unwrap()
    }

    fn with_local_partition<
        P: ReporterContext,
        Q,
        F: for<'r> FnOnce(
            Result<
                &mut MonolithicPartition<'r, P::Reporter>,
                &mut Self::ParallelPartition<P::Reporter>,
            >,
        ) -> Q,
    >(
        &mut self,
        reporter_context: P,
        inner: F,
    ) -> Q {
        if self.is_monolithic() {
            reporter_context.with_reporter(|reporter| {
                inner(Ok(&mut MonolithicPartition::from_reporter(reporter)))
            })
        } else {
            inner(Err(&mut MpiParallelPartition {
                world: self.world,
                _marker: PhantomData::<P::Reporter>,
            }))
        }
    }
}

pub struct MpiParallelPartition<R: Reporter> {
    world: SystemCommunicator,
    _marker: PhantomData<R>,
}

#[contract_trait]
impl<R: Reporter> ParallelPartition<R> for MpiParallelPartition<R> {
    type Partitioning = MpiPartitioning;

    fn get_partition_rank(&self) -> u32 {
        #[allow(clippy::cast_sign_loss)]
        {
            self.world.rank() as u32
        }
    }

    fn get_number_of_partitions(&self) -> NonZeroU32 {
        #[allow(clippy::cast_sign_loss)]
        NonZeroU32::new(self.world.size() as u32).unwrap()
    }
}

impl<R: Reporter> Partition<R> for MpiParallelPartition<R> {
    type Reporter = Self;

    fn get_reporter(&mut self) -> &mut Self::Reporter {
        self
    }
}

impl<R: Reporter> Reporter for MpiParallelPartition<R> {
    #[inline]
    fn report_event(&mut self, _event: &Event) {
        // TODO: Dump events to disk
    }

    #[inline]
    fn report_progress(&mut self, _remaining: u64) {
        // TODO: Report progress with some max frequency to the root, which can
        // pass it on
    }
}

impl<R: Reporter> EventFilter for MpiParallelPartition<R> {
    const REPORT_DISPERSAL: bool = R::REPORT_DISPERSAL;
    const REPORT_SPECIATION: bool = R::REPORT_SPECIATION;
}
