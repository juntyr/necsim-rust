use anyhow::Result;

use necsim_core::reporter::Reporter;
use necsim_partitioning_core::LocalPartition;

use crate::args::{Algorithm as AlgorithmArgs, CommonArgs, Scenario as ScenarioArgs};

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
pub fn simulate_with_logger<R: Reporter, P: LocalPartition<R>, V: FnOnce(), L: FnOnce()>(
    local_partition: Box<P>,
    common_args: CommonArgs,
    scenario: ScenarioArgs,
    algorithm: AlgorithmArgs,
    post_validation: V,
    pre_launch: L,
) -> Result<()> {
    #[cfg(any(
        feature = "rustcoalescence-algorithms-monolithic",
        feature = "rustcoalescence-algorithms-independent",
        feature = "rustcoalescence-algorithms-cuda"
    ))]
    {
        dispatch::simulate_with_logger(
            local_partition,
            common_args,
            scenario,
            algorithm,
            post_validation,
            pre_launch,
        )
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
        std::mem::drop(algorithm);
        std::mem::drop(post_validation);
        std::mem::drop(pre_launch);

        Err(anyhow::anyhow!(
            "rustcoalescence must be compiled to support at least one algorithm."
        ))
    }
}
