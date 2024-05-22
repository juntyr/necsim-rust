use serde::{Deserialize, Serialize};

use necsim_partitioning_core::partition::PartitionSize;

#[derive(Debug, Serialize, Deserialize)]
pub enum Partitioning {
    Monolithic(necsim_partitioning_monolithic::MonolithicPartitioning),
    #[cfg(feature = "necsim-partitioning-mpi")]
    #[serde(alias = "MPI")]
    Mpi(necsim_partitioning_mpi::MpiPartitioning),
}

impl Partitioning {
    // pub fn is_root(&self) -> bool {
    //     use necsim_partitioning_core::Partitioning;

    //     match self {
    //         Self::Monolithic(partitioning) => partitioning.is_root(),
    //         #[cfg(feature = "necsim-partitioning-mpi")]
    //         Self::Mpi(partitioning) => partitioning.is_root(),
    //     }
    // }

    pub fn get_size(&self) -> PartitionSize {
        use necsim_partitioning_core::Partitioning;

        match self {
            Self::Monolithic(partitioning) => partitioning.get_size(),
            #[cfg(feature = "necsim-partitioning-mpi")]
            Self::Mpi(partitioning) => partitioning.get_size(),
        }
    }

    pub fn get_event_log_check(&self) -> (anyhow::Result<()>, anyhow::Result<()>) {
        match self {
            Self::Monolithic(_) => (Ok(()), Ok(())),
            #[cfg(feature = "necsim-partitioning-mpi")]
            Self::Mpi(_) => (
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
