#![deny(clippy::pedantic)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(stmt_expr_attributes)]

#[macro_use]
extern crate contracts;

use std::{fmt, num::NonZeroU32};

use mpi::{
    environment::Universe,
    topology::{Communicator, Rank, SystemCommunicator},
    Tag,
};
use serde::Deserialize;
use serde_derive_state::DeserializeState;
use serde_state::{DeserializeState, Deserializer};
use thiserror::Error;

use necsim_core::reporter::Reporter;
use necsim_core_bond::Partition;

use necsim_impls_std::event_log::recorder::EventLogRecorder;
use necsim_partitioning_core::{context::ReporterContext, Partitioning};

mod partition;

pub use partition::{MpiLocalPartition, MpiParallelPartition, MpiRootPartition};

#[derive(Error, Debug)]
pub enum MpiPartitioningError {
    #[error("MPI has already been initialised.")]
    AlreadyInitialised,
    #[error("MPI must be initialised with at least two partitions.")]
    NoParallelism,
}

#[derive(Error, Debug)]
pub enum MpiLocalPartitionError {
    #[error("MPI partitioning must be used with an event log.")]
    MissingEventLog,
}

static mut MPI_UNIVERSE: Option<Universe> = None;

pub struct MpiPartitioning {
    world: SystemCommunicator,
}

impl fmt::Debug for MpiPartitioning {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(MpiPartitioning))
            .field("world", &self.get_partition().size().get())
            .finish()
    }
}

impl<'de> Deserialize<'de> for MpiPartitioning {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let partitioning = Self::initialise().map_err(serde::de::Error::custom)?;

        let _raw = MpiPartitioningRaw::deserialize_state(
            &mut partitioning.get_partition().size(),
            deserializer,
        )?;

        Ok(partitioning)
    }
}

impl MpiPartitioning {
    const MPI_MIGRATION_TAG: Tag = 1;
    const MPI_PROGRESS_TAG: Tag = 0;
    const ROOT_RANK: Rank = 0;

    /// # Errors
    ///
    /// Returns `AlreadyInitialised` if MPI was already initialised.
    /// Returns `NoParallelism` if the MPI world only consists of one or less
    ///  partitions.
    pub fn initialise() -> Result<Self, MpiPartitioningError> {
        let world = if let Some(universe) = unsafe { &MPI_UNIVERSE } {
            universe.world()
        } else if let Some(universe) = mpi::initialize() {
            let universe = unsafe { MPI_UNIVERSE.insert(universe) };

            universe.world()
        } else {
            return Err(MpiPartitioningError::AlreadyInitialised);
        };

        if world.size() > 1 {
            Ok(Self { world })
        } else {
            Err(MpiPartitioningError::NoParallelism)
        }
    }

    #[allow(clippy::unused_self)]
    pub fn update_message_buffer_size(&mut self, size: usize) {
        if let Some(universe) = unsafe { MPI_UNIVERSE.as_mut() } {
            universe.set_buffer_size(size);
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

    fn get_partition(&self) -> Partition {
        #[allow(clippy::cast_sign_loss)]
        let rank = self.world.rank() as u32;
        #[allow(clippy::cast_sign_loss)]
        let size = unsafe { NonZeroU32::new_unchecked(self.world.size() as u32) };

        unsafe { Partition::new_unchecked(rank, size) }
    }

    /// # Errors
    ///
    /// Returns `AlreadyInitialised` if MPI was already initialised.
    /// Returns `NoParallelism` if the MPI world only consists of one or less
    ///  partitions.
    /// Returns `MissingEventLog` if the local partition is non-monolithic and
    ///  the `event_log` is `None`.
    fn into_local_partition<R: Reporter, P: ReporterContext<Reporter = R>>(
        self,
        reporter_context: P,
        event_log: Self::Auxiliary,
    ) -> anyhow::Result<Self::LocalPartition<R>> {
        // Only one MPI universe can exist, and only one can be used to
        //  construct the local MPI partition
        let universe = match unsafe { MPI_UNIVERSE.take() } {
            Some(universe) => universe,
            None => anyhow::bail!(MpiPartitioningError::AlreadyInitialised),
        };

        if self.world.size() <= 1 {
            anyhow::bail!(MpiPartitioningError::NoParallelism);
        }

        let event_log = match event_log {
            Some(event_log) => event_log,
            None => anyhow::bail!(MpiLocalPartitionError::MissingEventLog),
        };

        if self.world.rank() == MpiPartitioning::ROOT_RANK {
            Ok(MpiLocalPartition::Root(Box::new(MpiRootPartition::new(
                universe,
                self.world,
                reporter_context.try_build()?,
                event_log,
            ))))
        } else {
            Ok(MpiLocalPartition::Parallel(Box::new(
                MpiParallelPartition::new(universe, self.world, event_log),
            )))
        }
    }
}

#[derive(DeserializeState)]
#[serde(rename = "MpiPartitioning")]
#[serde(deny_unknown_fields)]
#[serde(deserialize_state = "NonZeroU32")]
#[allow(dead_code)]
struct MpiPartitioningRaw {
    #[serde(deserialize_state_with = "deserialize_state_mpi_world")]
    #[serde(default)]
    world: Option<NonZeroU32>,
}

fn deserialize_state_mpi_world<'de, D>(
    mpi_world: &mut NonZeroU32,
    deserializer: D,
) -> Result<Option<NonZeroU32>, D::Error>
where
    D: Deserializer<'de>,
{
    let maybe_world = Option::<NonZeroU32>::deserialize(deserializer)?;

    match maybe_world {
        None => Ok(None),
        Some(world) if world == *mpi_world => Ok(Some(world)),
        Some(_) => Err(serde::de::Error::custom(format!(
            "mismatch with MPI world size of {}",
            mpi_world
        ))),
    }
}
