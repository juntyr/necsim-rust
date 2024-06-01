#![deny(clippy::pedantic)]

use std::{
    fmt,
    num::Wrapping,
    ops::ControlFlow,
    sync::{mpsc::sync_channel, Arc, Barrier},
    time::Duration,
};

use anyhow::Context;
use humantime_serde::re::humantime::format_duration;
use necsim_core_bond::PositiveF64;
use serde::{ser::SerializeStruct, Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use necsim_core::reporter::{
    boolean::{False, True},
    FilteredReporter, Reporter,
};

use necsim_impls_std::event_log::recorder::EventLogRecorder;
use necsim_partitioning_core::{
    partition::PartitionSize,
    reporter::{FinalisableReporter, ReporterContext},
    Data, Partitioning,
};

mod partition;
mod vote;

pub use partition::ThreadsLocalPartition;
use vote::Vote;

use crate::vote::AsyncVote;

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
    num_threads: PartitionSize,
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
            .field("num_threads", &self.num_threads.get())
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
        args.serialize_field("threads", &self.num_threads)?;
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
            num_threads: raw.num_threads,
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

impl Partitioning for ThreadsPartitioning {
    type Auxiliary = Option<EventLogRecorder>;
    type FinalisableReporter<R: Reporter> = FinalisableThreadsReporter<R>;
    type LocalPartition<R: Reporter> = ThreadsLocalPartition<R>;

    fn get_size(&self) -> PartitionSize {
        self.num_threads
    }

    #[allow(clippy::too_many_lines)]
    /// # Errors
    ///
    /// Returns `MissingEventLog` if the local partition is non-monolithic and
    ///  the `event_log` is `None`.
    /// Returns `InvalidEventSubLog` if creating a sub-`event_log` failed.
    fn with_local_partition<
        R: Reporter,
        P: ReporterContext<Reporter = R>,
        A: Data,
        Q: Data + serde::Serialize + serde::de::DeserializeOwned,
    >(
        self,
        reporter_context: P,
        event_log: Self::Auxiliary,
        args: A,
        inner: fn(&mut Self::LocalPartition<R>, A) -> Q,
        fold: fn(Q, Q) -> Q,
    ) -> anyhow::Result<(Q, Self::FinalisableReporter<R>)> {
        // TODO: add support for multithread live reporting
        let Some(event_log) = event_log else {
            anyhow::bail!(ThreadsLocalPartitionError::MissingEventLog)
        };

        let mut progress_reporter: FilteredReporter<R, False, False, True> =
            reporter_context.try_build()?;
        let (progress_sender, progress_receiver) = sync_channel(self.num_threads.get() as usize);
        let progress_channels = self
            .num_threads
            .partitions()
            .map(|_| progress_sender.clone())
            .collect::<Vec<_>>();
        std::mem::drop(progress_sender);

        let vote_any = Vote::new(self.num_threads.get() as usize);
        let vote_min_time =
            Vote::new_with_dummy(self.num_threads.get() as usize, (PositiveF64::one(), 0));
        let vote_termination =
            AsyncVote::new_with_dummy(self.num_threads.get() as usize, ControlFlow::Continue(()));

        let (emigration_channels, immigration_channels): (Vec<_>, Vec<_>) = self
            .num_threads
            .partitions()
            .map(|_| sync_channel(self.num_threads.get() as usize))
            .unzip();

        let event_logs = self
            .num_threads
            .partitions()
            .map(|partition| {
                event_log
                    .new_child_log(&partition.rank().to_string())
                    .context(ThreadsLocalPartitionError::InvalidEventSubLog)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let sync_barrier = Arc::new(Barrier::new(self.num_threads.get() as usize));
        let args = self
            .num_threads
            .partitions()
            .map(|_| args.clone())
            .collect::<Vec<_>>();

        let result = std::thread::scope(|scope| {
            let vote_any = &vote_any;
            let vote_min_time = &vote_min_time;
            let vote_termination = &vote_termination;
            let emigration_channels = emigration_channels.as_slice();
            let sync_barrier = &sync_barrier;

            let thread_handles = self
                .num_threads
                .partitions()
                .zip(immigration_channels)
                .zip(event_logs)
                .zip(progress_channels)
                .zip(args)
                .map(
                    |((((partition, immigration_channel), event_log), progress_channel), args)| {
                        scope.spawn(move || {
                            let mut local_partition = ThreadsLocalPartition::<R>::new(
                                partition,
                                vote_any,
                                vote_min_time,
                                vote_termination,
                                emigration_channels,
                                immigration_channel,
                                self.migration_interval,
                                event_log,
                                progress_channel,
                                self.progress_interval,
                                sync_barrier,
                            );

                            inner(&mut local_partition, args)
                        })
                    },
                )
                .collect::<Vec<_>>();

            let mut progress_remaining =
                vec![0; self.num_threads.get() as usize].into_boxed_slice();
            for (remaining, rank) in progress_receiver {
                progress_remaining[rank as usize] = remaining;
                progress_reporter.report_progress(
                    (&progress_remaining
                        .iter()
                        .map(|r| Wrapping(*r))
                        .sum::<Wrapping<u64>>()
                        .0)
                        .into(),
                );
            }

            let mut folded_result = None;
            for handle in thread_handles {
                let result = match handle.join() {
                    Ok(result) => result,
                    Err(payload) => std::panic::resume_unwind(payload),
                };
                folded_result = Some(match folded_result.take() {
                    Some(acc) => fold(acc, result),
                    None => result,
                });
            }
            folded_result.expect("at least one threads partitioning result")
        });

        Ok((
            result,
            FinalisableThreadsReporter {
                reporter: progress_reporter,
            },
        ))
    }
}

pub struct FinalisableThreadsReporter<R: Reporter> {
    reporter: FilteredReporter<R, False, False, True>,
}

impl<R: Reporter> FinalisableReporter for FinalisableThreadsReporter<R> {
    fn finalise(self) {
        self.reporter.finalise();
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
