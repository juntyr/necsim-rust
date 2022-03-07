#![deny(clippy::pedantic)]
#![feature(generic_associated_types)]

#[macro_use]
extern crate contracts;

use std::{fmt, mem::ManuallyDrop, num::NonZeroU32, time::Duration};

use anyhow::Context;
use humantime_serde::re::humantime::format_duration;
use mpi::{
    environment::Universe,
    topology::{Communicator, Rank, SystemCommunicator},
    Tag,
};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use serde_derive_state::DeserializeState;
use serde_state::{DeserializeState, Deserializer};
use thiserror::Error;

use necsim_core::{lineage::MigratingLineage, reporter::Reporter};

use necsim_impls_std::event_log::recorder::EventLogRecorder;
use necsim_partitioning_core::{context::ReporterContext, partition::Partition, Partitioning};

mod partition;
mod request;

pub use partition::{MpiLocalPartition, MpiParallelPartition, MpiRootPartition};
use request::{reduce_scope, DataOrRequest};

#[derive(Error, Debug)]
pub enum MpiPartitioningError {
    #[error("MPI has already been initialised.")]
    AlreadyInitialised,
    #[error("MPI must be initialised with at least two partitions.")]
    NoParallelism,
}

#[derive(Error, Debug)]
pub enum MpiLocalPartitionError {
    #[error("MPI partitioning requires an event log.")]
    MissingEventLog,
    #[error("Failed to create the event sub-log.")]
    InvalidEventSubLog,
}

pub struct MpiPartitioning {
    universe: ManuallyDrop<Universe>,
    world: SystemCommunicator,
    migration_interval: Duration,
    progress_interval: Duration,
}

impl fmt::Debug for MpiPartitioning {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct FormattedDuration(Duration);

        impl fmt::Debug for FormattedDuration {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str(&format_duration(self.0).to_string())
            }
        }

        fmt.debug_struct(stringify!(MpiPartitioning))
            .field("world", &self.get_partition().size().get())
            .field(
                "migration_interval",
                &FormattedDuration(self.migration_interval),
            )
            .field(
                "progress_interval",
                &FormattedDuration(self.progress_interval),
            )
            .finish()
    }
}

impl Serialize for MpiPartitioning {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut args = serializer.serialize_struct(stringify!(MpiPartitioning), 3)?;
        args.serialize_field("world", &self.get_partition().size())?;
        args.serialize_field(
            "migration",
            &format_duration(self.migration_interval).to_string(),
        )?;
        args.serialize_field(
            "progress",
            &format_duration(self.progress_interval).to_string(),
        )?;
        args.end()
    }
}

impl<'de> Deserialize<'de> for MpiPartitioning {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut partitioning = Self::initialise().map_err(serde::de::Error::custom)?;

        let raw = MpiPartitioningRaw::deserialize_state(
            &mut partitioning.get_partition().size(),
            deserializer,
        )?;

        partitioning.set_migration_interval(raw.migration_interval);
        partitioning.set_progress_interval(raw.progress_interval);

        Ok(partitioning)
    }
}

impl MpiPartitioning {
    #[doc(hidden)]
    const MPI_DEFAULT_MIGRATION_INTERVAL: Duration = Duration::from_millis(100_u64);
    #[doc(hidden)]
    const MPI_DEFAULT_PROGRESS_INTERVAL: Duration = Duration::from_millis(100_u64);
    const MPI_MIGRATION_TAG: Tag = 1;
    const MPI_PROGRESS_TAG: Tag = 0;
    const ROOT_RANK: Rank = 0;

    /// # Errors
    ///
    /// Returns `AlreadyInitialised` if MPI was already initialised.
    /// Returns `NoParallelism` if the MPI world only consists of one or less
    ///  partitions.
    pub fn initialise() -> Result<Self, MpiPartitioningError> {
        let universe =
            ManuallyDrop::new(mpi::initialize().ok_or(MpiPartitioningError::AlreadyInitialised)?);
        let world = universe.world();

        if world.size() > 1 {
            Ok(Self {
                universe,
                world,
                migration_interval: Self::MPI_DEFAULT_MIGRATION_INTERVAL,
                progress_interval: Self::MPI_DEFAULT_PROGRESS_INTERVAL,
            })
        } else {
            Err(MpiPartitioningError::NoParallelism)
        }
    }

    pub fn set_migration_interval(&mut self, migration_interval: Duration) {
        self.migration_interval = migration_interval;
    }

    pub fn set_progress_interval(&mut self, progress_interval: Duration) {
        self.progress_interval = progress_interval;
    }
}

#[contract_trait]
impl Partitioning for MpiPartitioning {
    type Auxiliary = Option<EventLogRecorder>;
    type LocalPartition<'p, R: Reporter> = MpiLocalPartition<'p, R>;

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
    /// Returns `MissingEventLog` if the local partition is non-monolithic and
    ///  the `event_log` is `None`.
    /// Returns `InvalidEventSubLog` if creating a sub-`event_log` failed.
    fn with_local_partition<
        R: Reporter,
        P: ReporterContext<Reporter = R>,
        F: for<'p> FnOnce(Self::LocalPartition<'p, R>) -> Q,
        Q,
    >(
        self,
        reporter_context: P,
        event_log: Self::Auxiliary,
        inner: F,
    ) -> anyhow::Result<Q> {
        let event_log = match event_log {
            Some(event_log) => event_log,
            None => anyhow::bail!(MpiLocalPartitionError::MissingEventLog),
        };

        let mut directory = event_log.directory().to_owned();
        directory.push(self.world.rank().to_string());

        let event_log = event_log
            .r#move(&directory)
            .and_then(EventLogRecorder::assert_empty)
            .context(MpiLocalPartitionError::InvalidEventSubLog)?;

        let mut mpi_local_global_continue = (false, false);
        let mut mpi_local_remaining = 0_u64;

        #[allow(clippy::cast_sign_loss)]
        let world_size = self.world.size() as usize;

        let mut mpi_migration_buffers: Vec<Vec<MigratingLineage>> = Vec::with_capacity(world_size);
        mpi_migration_buffers.resize_with(world_size, Vec::new);

        mpi::request::scope(|scope| {
            let scope = reduce_scope(scope);

            let mpi_local_global_continue =
                DataOrRequest::new(&mut mpi_local_global_continue, scope);
            let mpi_local_remaining = DataOrRequest::new(&mut mpi_local_remaining, scope);
            let mpi_migration_buffers = mpi_migration_buffers
                .iter_mut()
                .map(|buffer| DataOrRequest::new(buffer, scope))
                .collect::<Vec<_>>()
                .into_boxed_slice();

            let local_partition = if self.world.rank() == MpiPartitioning::ROOT_RANK {
                MpiLocalPartition::Root(Box::new(MpiRootPartition::new(
                    ManuallyDrop::into_inner(self.universe),
                    mpi_local_global_continue,
                    mpi_migration_buffers,
                    reporter_context.try_build()?,
                    event_log,
                    self.migration_interval,
                    self.progress_interval,
                )))
            } else {
                MpiLocalPartition::Parallel(Box::new(MpiParallelPartition::new(
                    ManuallyDrop::into_inner(self.universe),
                    mpi_local_global_continue,
                    mpi_local_remaining,
                    mpi_migration_buffers,
                    event_log,
                    self.migration_interval,
                    self.progress_interval,
                )))
            };

            Ok(inner(local_partition))
        })
    }
}

#[derive(DeserializeState)]
#[serde(rename = "MpiPartitioning")]
#[serde(deny_unknown_fields)]
#[serde(deserialize_state = "NonZeroU32")]
#[serde(default)]
#[allow(dead_code)]
struct MpiPartitioningRaw {
    #[serde(deserialize_state_with = "deserialize_state_mpi_world")]
    world: Option<NonZeroU32>,
    #[serde(alias = "migration")]
    #[serde(with = "humantime_serde")]
    migration_interval: Duration,
    #[serde(alias = "progress")]
    #[serde(with = "humantime_serde")]
    progress_interval: Duration,
}

impl Default for MpiPartitioningRaw {
    fn default() -> Self {
        Self {
            world: None,
            migration_interval: MpiPartitioning::MPI_DEFAULT_MIGRATION_INTERVAL,
            progress_interval: MpiPartitioning::MPI_DEFAULT_PROGRESS_INTERVAL,
        }
    }
}

fn deserialize_state_mpi_world<'de, D: Deserializer<'de>>(
    mpi_world: &mut NonZeroU32,
    deserializer: D,
) -> Result<Option<NonZeroU32>, D::Error> {
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
