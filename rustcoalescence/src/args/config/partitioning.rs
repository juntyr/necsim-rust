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
}

impl Default for Partitioning {
    fn default() -> Self {
        Self::Monolithic(necsim_partitioning_monolithic::MonolithicPartitioning::default())
    }
}
