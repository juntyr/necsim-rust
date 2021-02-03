#![deny(clippy::pedantic)]

#[macro_use]
extern crate contracts;

use core::num::NonZeroU32;
use std::marker::PhantomData;

use mpi::{
    environment::Universe,
    topology::{Communicator, SystemCommunicator},
};

use necsim_core::reporter::Reporter;

use necsim_impls_no_std::{
    partitioning::{MonolithicPartition, ParallelPartition, Partition, Partitioning},
    reporter::ReporterContext,
};

pub struct MpiPartitioning<R: Reporter> {
    universe: Universe,
    world: SystemCommunicator,
    _marker: PhantomData<R>,
}

impl<R: Reporter> MpiPartitioning<R> {
    #[must_use]
    pub fn initialise() -> Option<Self> {
        mpi::initialize().map(|universe| Self {
            world: universe.world(),
            universe,
            _marker: PhantomData::<R>,
        })
    }

    pub fn update_message_buffer_size(&mut self, size: usize) {
        if !self.is_monolithic() {
            self.universe.set_buffer_size(size)
        }
    }
}

#[contract_trait]
impl<R: Reporter> Partitioning<R> for MpiPartitioning<R> {
    type ParallelPartition = MpiParallelPartition<R>;

    fn is_monolithic(&self) -> bool {
        self.world.size() <= 0
    }

    fn get_number_of_partitions(&self) -> NonZeroU32 {
        #[allow(clippy::cast_sign_loss)]
        NonZeroU32::new(self.world.size() as u32).unwrap()
    }

    fn with_local_partition<
        P: ReporterContext<Reporter = R>,
        Q,
        F: for<'r> FnOnce(
            Result<&mut MonolithicPartition<'r, P::Reporter>, &mut Self::ParallelPartition>,
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
                _marker: PhantomData::<R>,
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
    type Partitioning = MpiPartitioning<R>;

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
    fn get_reporter(&mut self) -> &mut R {
        unimplemented!("Progress should be reported live, but events should be dumped to disk")
    }
}
