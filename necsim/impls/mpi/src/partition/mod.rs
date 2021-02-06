use std::num::NonZeroU32;

use necsim_core::{
    event::Event,
    reporter::{EventFilter, Reporter},
};

use necsim_impls_no_std::{
    partitioning::{LocalPartition, MonolithicLocalPartition},
    reporter::ReporterContext,
};

mod parallel;
mod root;

pub use parallel::MpiParallelPartition;
pub use root::MpiRootPartition;

#[allow(clippy::module_name_repetitions)]
pub enum MpiLocalPartition<P: ReporterContext> {
    Monolithic(MonolithicLocalPartition<P>),
    Root(MpiRootPartition<P>),
    Parallel(MpiParallelPartition<P>),
}

#[contract_trait]
impl<P: ReporterContext> LocalPartition<P> for MpiLocalPartition<P> {
    type Reporter = Self;

    fn get_reporter(&mut self) -> &mut Self::Reporter {
        self
    }

    fn is_root(&self) -> bool {
        match self {
            Self::Monolithic(partition) => partition.is_root(),
            Self::Root(partition) => partition.is_root(),
            Self::Parallel(partition) => partition.is_root(),
        }
    }

    fn get_partition_rank(&self) -> u32 {
        match self {
            Self::Monolithic(partition) => partition.get_partition_rank(),
            Self::Root(partition) => partition.get_partition_rank(),
            Self::Parallel(partition) => partition.get_partition_rank(),
        }
    }

    fn get_number_of_partitions(&self) -> NonZeroU32 {
        match self {
            Self::Monolithic(partition) => partition.get_number_of_partitions(),
            Self::Root(partition) => partition.get_number_of_partitions(),
            Self::Parallel(partition) => partition.get_number_of_partitions(),
        }
    }

    fn reduce_global_time_steps(&self, local_time: f64, local_steps: u64) -> (f64, u64) {
        match self {
            Self::Monolithic(partition) => {
                partition.reduce_global_time_steps(local_time, local_steps)
            },
            Self::Root(partition) => partition.reduce_global_time_steps(local_time, local_steps),
            Self::Parallel(partition) => {
                partition.reduce_global_time_steps(local_time, local_steps)
            },
        }
    }

    fn wait_for_termination(&mut self) -> bool {
        match self {
            Self::Monolithic(partition) => partition.wait_for_termination(),
            Self::Root(partition) => partition.wait_for_termination(),
            Self::Parallel(partition) => partition.wait_for_termination(),
        }
    }
}

impl<P: ReporterContext> Reporter for MpiLocalPartition<P> {
    #[inline]
    fn report_event(&mut self, event: &Event) {
        match self {
            Self::Monolithic(partition) => partition.get_reporter().report_event(event),
            Self::Root(partition) => partition.get_reporter().report_event(event),
            Self::Parallel(partition) => partition.get_reporter().report_event(event),
        }
    }

    #[inline]
    fn report_progress(&mut self, remaining: u64) {
        match self {
            Self::Monolithic(partition) => partition.get_reporter().report_progress(remaining),
            Self::Root(partition) => partition.get_reporter().report_progress(remaining),
            Self::Parallel(partition) => partition.get_reporter().report_progress(remaining),
        }
    }
}

impl<P: ReporterContext> EventFilter for MpiLocalPartition<P> {
    const REPORT_DISPERSAL: bool = P::Reporter::REPORT_DISPERSAL;
    const REPORT_SPECIATION: bool = P::Reporter::REPORT_SPECIATION;
}
