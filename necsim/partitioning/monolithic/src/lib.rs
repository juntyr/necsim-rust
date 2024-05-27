#![deny(clippy::pedantic)]

#[macro_use]
extern crate contracts;

use std::{fmt, ops::ControlFlow};

use anyhow::Context;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use necsim_core::{
    impl_report,
    lineage::MigratingLineage,
    reporter::{boolean::True, Reporter},
};
use necsim_core_bond::PositiveF64;

use necsim_partitioning_core::{
    context::ReporterContext,
    iterator::ImmigrantPopIterator,
    partition::{Partition, PartitionSize},
    LocalPartition, MigrationMode, Partitioning,
};

use necsim_impls_std::event_log::recorder::EventLogRecorder;

pub mod live;
pub mod recorded;

#[allow(clippy::module_name_repetitions)]
#[derive(Default)]
pub struct MonolithicPartitioning(());

impl fmt::Debug for MonolithicPartitioning {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(MonolithicPartitioning))
            .finish()
    }
}

impl Serialize for MonolithicPartitioning {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_unit()
    }
}

impl<'de> Deserialize<'de> for MonolithicPartitioning {
    fn deserialize<D: Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::default())
    }
}

#[contract_trait]
impl Partitioning for MonolithicPartitioning {
    type Auxiliary = Option<EventLogRecorder>;
    type LocalPartition<'p, R: Reporter> = MonolithicLocalPartition<R>;

    fn get_size(&self) -> PartitionSize {
        PartitionSize::MONOLITHIC
    }

    /// # Errors
    ///
    /// Returns an error if the provided event log is not empty.
    fn with_local_partition<R: Reporter, P: ReporterContext<Reporter = R>, A, Q>(
        self,
        reporter_context: P,
        event_log: Self::Auxiliary,
        args: A,
        inner: for<'p> fn(Self::LocalPartition<'p, R>, A) -> Q,
        _fold: fn(Q, Q) -> Q,
    ) -> anyhow::Result<Q> {
        let local_partition = if let Some(event_log) = event_log {
            MonolithicLocalPartition::Recorded(Box::new(
                recorded::RecordedMonolithicLocalPartition::try_from_context_and_recorder(
                    reporter_context,
                    event_log
                        .assert_empty()
                        .context("Failed to create the event log.")?,
                )?,
            ))
        } else {
            MonolithicLocalPartition::Live(Box::new(
                live::LiveMonolithicLocalPartition::try_from_context(reporter_context)?,
            ))
        };

        Ok(inner(local_partition, args))
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum MonolithicLocalPartition<R: Reporter> {
    Live(Box<live::LiveMonolithicLocalPartition<R>>),
    Recorded(Box<recorded::RecordedMonolithicLocalPartition<R>>),
}

#[contract_trait]
impl<'p, R: Reporter> LocalPartition<'p, R> for MonolithicLocalPartition<R> {
    type ImmigrantIterator<'a> = ImmigrantPopIterator<'a> where 'p: 'a, R: 'a;
    // pessimistic
    type IsLive = True;
    type Reporter = Self;

    fn get_reporter(&mut self) -> &mut Self::Reporter {
        self
    }

    fn is_root(&self) -> bool {
        match self {
            Self::Live(partition) => partition.is_root(),
            Self::Recorded(partition) => partition.is_root(),
        }
    }

    fn get_partition(&self) -> Partition {
        match self {
            Self::Live(partition) => partition.get_partition(),
            Self::Recorded(partition) => partition.get_partition(),
        }
    }

    fn migrate_individuals<'a, E: Iterator<Item = (u32, MigratingLineage)>>(
        &'a mut self,
        emigrants: &mut E,
        emigration_mode: MigrationMode,
        immigration_mode: MigrationMode,
    ) -> Self::ImmigrantIterator<'a>
    where
        'p: 'a,
    {
        match self {
            Self::Live(partition) => {
                partition.migrate_individuals(emigrants, emigration_mode, immigration_mode)
            },
            Self::Recorded(partition) => {
                partition.migrate_individuals(emigrants, emigration_mode, immigration_mode)
            },
        }
    }

    fn reduce_vote_any(&mut self, vote: bool) -> bool {
        match self {
            Self::Live(partition) => partition.reduce_vote_any(vote),
            Self::Recorded(partition) => partition.reduce_vote_any(vote),
        }
    }

    fn reduce_vote_min_time(
        &mut self,
        local_time: PositiveF64,
    ) -> Result<PositiveF64, PositiveF64> {
        match self {
            Self::Live(partition) => partition.reduce_vote_min_time(local_time),
            Self::Recorded(partition) => partition.reduce_vote_min_time(local_time),
        }
    }

    fn wait_for_termination(&mut self) -> ControlFlow<(), ()> {
        match self {
            Self::Live(partition) => partition.wait_for_termination(),
            Self::Recorded(partition) => partition.wait_for_termination(),
        }
    }

    fn report_progress_sync(&mut self, remaining: u64) {
        match self {
            Self::Live(partition) => partition.report_progress_sync(remaining),
            Self::Recorded(partition) => partition.report_progress_sync(remaining),
        }
    }

    fn finalise_reporting(self) {
        match self {
            Self::Live(partition) => partition.finalise_reporting(),
            Self::Recorded(partition) => partition.finalise_reporting(),
        }
    }
}

impl<R: Reporter> Reporter for MonolithicLocalPartition<R> {
    impl_report!(speciation(&mut self, speciation: MaybeUsed<R::ReportSpeciation>) {
        match self {
            Self::Live(partition) => partition.get_reporter().report_speciation(
                speciation.into()
            ),
            Self::Recorded(partition) => partition.get_reporter().report_speciation(
                speciation.into()
            ),
        }
    });

    impl_report!(dispersal(&mut self, dispersal: MaybeUsed<R::ReportDispersal>) {
        match self {
            Self::Live(partition) => partition.get_reporter().report_dispersal(
                dispersal.into()
            ),
            Self::Recorded(partition) => partition.get_reporter().report_dispersal(
                dispersal.into()
            ),
        }
    });

    impl_report!(progress(&mut self, progress: MaybeUsed<R::ReportProgress>) {
        match self {
            Self::Live(partition) => partition.get_reporter().report_progress(
                progress.into()
            ),
            Self::Recorded(partition) => partition.get_reporter().report_progress(
                progress.into()
            ),
        }
    });
}
