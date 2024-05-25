#![deny(clippy::pedantic)]

#[macro_use]
extern crate contracts;

use std::{fmt, sync::mpsc::sync_channel, time::Duration};

use anyhow::Context;
use humantime_serde::re::humantime::format_duration;
use necsim_core_bond::{NonNegativeF64, PositiveF64};
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use necsim_core::reporter::Reporter;

use necsim_impls_std::event_log::recorder::EventLogRecorder;
use necsim_partitioning_core::{context::ReporterContext, partition::PartitionSize, Partitioning};

mod partition;
mod vote;

pub use partition::{ThreadsLocalPartition, ThreadsParallelPartition, ThreadsRootPartition};
use vote::Vote;

#[derive(Error, Debug)]
pub enum ThreadsPartitioningError {
    #[error("Threads partitioning must be initialised with at least two partitions.")]
    NoParallelism,
}

#[derive(Error, Debug)]
pub enum ThreadsLocalPartitionError {
    #[error("Threads partitioning requires an event log.")]
    MissingEventLog,
    #[error("Failed to create the event sub-log.")]
    InvalidEventSubLog,
}

pub struct ThreadsPartitioning {
    size: PartitionSize,
    migration_interval: Duration,
    progress_interval: Duration,
}

impl fmt::Debug for ThreadsPartitioning {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct FormattedDuration(Duration);

        impl fmt::Debug for FormattedDuration {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                fmt.write_str(&format_duration(self.0).to_string())
            }
        }

        fmt.debug_struct(stringify!(ThreadsPartitioning))
            .field("size", &self.get_size().get())
            .field(
                "migration_interval",
                &FormattedDuration(self.migration_interval),
            )
            .field(
                "progress_interval",
                &FormattedDuration(self.progress_interval),
            )
            .finish_non_exhaustive()
    }
}

impl Serialize for ThreadsPartitioning {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut args = serializer.serialize_struct(stringify!(ThreadsPartitioning), 3)?;
        args.serialize_field("size", &self.get_size())?;
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

impl<'de> Deserialize<'de> for ThreadsPartitioning {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = ThreadsPartitioningRaw::deserialize(deserializer)?;

        Ok(Self {
            size: raw.num_threads,
            migration_interval: raw.migration_interval,
            progress_interval: raw.progress_interval,
        })
    }
}

impl ThreadsPartitioning {
    const DEFAULT_MIGRATION_INTERVAL: Duration = Duration::from_millis(100_u64);
    const DEFAULT_PROGRESS_INTERVAL: Duration = Duration::from_millis(100_u64);

    pub fn set_migration_interval(&mut self, migration_interval: Duration) {
        self.migration_interval = migration_interval;
    }

    pub fn set_progress_interval(&mut self, progress_interval: Duration) {
        self.progress_interval = progress_interval;
    }
}

#[contract_trait]
impl Partitioning for ThreadsPartitioning {
    type Auxiliary = Option<EventLogRecorder>;
    type LocalPartition<'p, R: Reporter> = ThreadsLocalPartition<'p, R>;

    fn get_size(&self) -> PartitionSize {
        self.size
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
        _reporter_context: P,
        event_log: Self::Auxiliary,
        _inner: F,
    ) -> anyhow::Result<Q> {
        let vote_any = Vote::new(self.size.get() as usize);
        let vote_min_time = Vote::new_with_dummy(self.size.get() as usize, (PositiveF64::one(), 0));
        let vote_time_steps =
            Vote::new_with_dummy(self.size.get() as usize, (NonNegativeF64::zero(), 0));

        let (emigration_channels, immigration_channels): (Vec<_>, Vec<_>) = self
            .size
            .partitions()
            .map(|_| sync_channel(self.size.get() as usize))
            .unzip();

        // TODO: add support for multithread live reporting
        let Some(event_log) = event_log else {
            anyhow::bail!(ThreadsLocalPartitionError::MissingEventLog)
        };

        let event_logs = self
            .size
            .partitions()
            .map(|partition| {
                let mut directory = event_log.directory().to_owned();
                directory.push(partition.rank().to_string());

                event_log
                    .clone_move(directory)
                    .and_then(EventLogRecorder::assert_empty)
                    .context(ThreadsLocalPartitionError::InvalidEventSubLog)
            })
            .collect::<Result<Vec<_>, _>>()?;

        std::thread::scope(|scope| {
            let vote_any = &vote_any;
            let vote_min_time = &vote_min_time;
            let vote_time_steps = &vote_time_steps;
            let emigration_channels = emigration_channels.as_slice();

            for ((partition, immigration_channel), event_log) in self
                .size
                .partitions()
                .zip(immigration_channels)
                .zip(event_logs)
            {
                let thread_handle = scope.spawn::<_, ()>(move || {
                    let _local_partition = if partition.is_root() {
                        ThreadsLocalPartition::Root(Box::new(ThreadsRootPartition::<R>::new(
                            partition,
                            vote_any,
                            vote_min_time,
                            vote_time_steps,
                            emigration_channels,
                            immigration_channel,
                            self.migration_interval,
                            event_log,
                            self.progress_interval,
                        )))
                    } else {
                        ThreadsLocalPartition::Parallel(Box::new(
                            ThreadsParallelPartition::<R>::new(
                                partition,
                                vote_any,
                                vote_min_time,
                                vote_time_steps,
                                emigration_channels,
                                immigration_channel,
                                self.migration_interval,
                                event_log,
                                self.progress_interval,
                            ),
                        ))
                    };
                });

                // we don't need the thread result and implicitly propagate thread panics
                std::mem::drop(thread_handle);
            }
        });

        todo!()
    }
}

#[derive(Deserialize)]
#[serde(rename = "ThreadsPartitioning")]
#[serde(deny_unknown_fields)]
struct ThreadsPartitioningRaw {
    #[serde(alias = "n", alias = "threads")]
    num_threads: PartitionSize,
    #[serde(alias = "migration")]
    #[serde(with = "humantime_serde")]
    #[serde(default = "default_migration_interval")]
    migration_interval: Duration,
    #[serde(alias = "progress")]
    #[serde(with = "humantime_serde")]
    #[serde(default = "default_progress_interval")]
    progress_interval: Duration,
}

fn default_migration_interval() -> Duration {
    ThreadsPartitioning::DEFAULT_MIGRATION_INTERVAL
}

fn default_progress_interval() -> Duration {
    ThreadsPartitioning::DEFAULT_PROGRESS_INTERVAL
}
