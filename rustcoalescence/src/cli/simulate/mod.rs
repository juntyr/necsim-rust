use anyhow::Result;

use necsim_core::reporter::Reporter;
use necsim_impls_no_std::partitioning::LocalPartition;

use crate::args::{CommonArgs, Scenario as ScenarioArgs};

#[cfg(not(feature = "necsim-partitioning-mpi"))]
pub mod monolithic;
#[cfg(feature = "necsim-partitioning-mpi")]
pub mod mpi;

#[macro_use]
mod r#impl;

#[cfg(any(
    feature = "rustcoalescence-algorithms-monolithic",
    feature = "rustcoalescence-algorithms-independent",
    feature = "rustcoalescence-algorithms-cuda"
))]
mod dispatch;

#[allow(clippy::module_name_repetitions)]
pub fn simulate_with_logger<R: Reporter, P: LocalPartition<R>>(
    local_partition: Box<P>,
    common_args: CommonArgs,
    scenario: ScenarioArgs,
) -> Result<()> {
    #[cfg(any(
        feature = "rustcoalescence-algorithms-monolithic",
        feature = "rustcoalescence-algorithms-independent",
        feature = "rustcoalescence-algorithms-cuda"
    ))]
    {
        dispatch::simulate_with_logger(local_partition, common_args, scenario)
    }

    #[cfg(not(any(
        feature = "rustcoalescence-algorithms-monolithic",
        feature = "rustcoalescence-algorithms-independent",
        feature = "rustcoalescence-algorithms-cuda"
    )))]
    {
        std::mem::drop(local_partition);
        std::mem::drop(common_args);
        std::mem::drop(scenario);

        Err(anyhow::anyhow!(
            "rustcoalescence must be compiled to support at least one algorithm."
        ))
    }
}
