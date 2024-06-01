use necsim_core::{
    cogs::{MathsCore, RngCore},
    reporter::Reporter,
};
use necsim_impls_std::event_log::recorder::EventLogRecorder;
use rustcoalescence_algorithms::AlgorithmDispatch;
use rustcoalescence_scenarios::Scenario;
use serde::{Deserialize, Serialize};

use necsim_partitioning_core::partition::PartitionSize;

#[derive(Debug, Serialize, Deserialize)]
pub enum Partitioning {
    Monolithic(necsim_partitioning_monolithic::MonolithicPartitioning),
    #[cfg(feature = "mpi-partitioning")]
    #[serde(alias = "MPI")]
    Mpi(necsim_partitioning_mpi::MpiPartitioning),
    #[cfg(feature = "threads-partitioning")]
    Threads(necsim_partitioning_threads::ThreadsPartitioning),
}

impl Partitioning {
    pub fn get_size(&self) -> PartitionSize {
        use necsim_partitioning_core::Partitioning;

        match self {
            Self::Monolithic(partitioning) => partitioning.get_size(),
            #[cfg(feature = "mpi-partitioning")]
            Self::Mpi(partitioning) => partitioning.get_size(),
            #[cfg(feature = "threads-partitioning")]
            Self::Threads(partitioning) => partitioning.get_size(),
        }
    }

    pub fn get_event_log_check(&self) -> (anyhow::Result<()>, anyhow::Result<()>) {
        match self {
            Self::Monolithic(_) => (Ok(()), Ok(())),
            #[cfg(feature = "mpi-partitioning")]
            Self::Mpi(_) => (
                Err(anyhow::anyhow!(
                    necsim_partitioning_mpi::MpiLocalPartitionError::MissingEventLog
                )),
                Ok(()),
            ),
            #[cfg(feature = "threads-partitioning")]
            Self::Threads(_) => (
                Err(anyhow::anyhow!(
                    necsim_partitioning_mpi::MpiLocalPartitionError::MissingEventLog
                )),
                Ok(()),
            ),
        }
    }

    pub fn get_logical_partition_size<
        M: MathsCore,
        G: RngCore<M>,
        O: Scenario<M, G>,
        R: Reporter,
        A: AlgorithmDispatch<M, G, O, R>,
    >(
        &self,
        algorithm_args: &A::Arguments,
    ) -> PartitionSize {
        match self {
            Partitioning::Monolithic(partitioning) => {
                A::get_logical_partition_size(algorithm_args, partitioning)
            },
            #[cfg(feature = "mpi-partitioning")]
            Partitioning::Mpi(partitioning) => {
                A::get_logical_partition_size(algorithm_args, partitioning)
            },
            #[cfg(feature = "threads-partitioning")]
            Partitioning::Threads(partitioning) => {
                A::get_logical_partition_size(algorithm_args, partitioning)
            },
        }
    }

    pub fn will_report_live(&self, event_log: &Option<EventLogRecorder>) -> bool {
        // TODO: get this information from the partitioning
        match self {
            Partitioning::Monolithic(_) => event_log.is_none(),
            #[cfg(feature = "mpi-partitioning")]
            Partitioning::Mpi(_) => false,
            #[cfg(feature = "threads-partitioning")]
            Partitioning::Threads(_) => false,
        }
    }
}

impl Default for Partitioning {
    fn default() -> Self {
        Self::Monolithic(necsim_partitioning_monolithic::MonolithicPartitioning::default())
    }
}
