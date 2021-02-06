use array2d::Array2D;

use crate::{partitioning::LocalPartition, reporter::ReporterContext};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
#[allow(clippy::module_name_repetitions, clippy::too_many_arguments)]
pub trait InMemorySimulation {
    type Error;
    type AuxiliaryArguments;

    #[debug_requires(
        (0.0_f64..=1.0_f64).contains(&speciation_probability_per_generation),
        "0.0 <= speciation_probability_per_generation <= 1.0"
    )]
    #[debug_requires(
        (0.0_f64..=1.0_f64).contains(&sample_percentage),
        "0.0 <= sample_percentage <= 1.0"
    )]
    fn simulate<R: ReporterContext, P: LocalPartition<R>>(
        habitat: &Array2D<u32>,
        dispersal: &Array2D<f64>,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        local_partition: &mut P,
        auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error>;
}
