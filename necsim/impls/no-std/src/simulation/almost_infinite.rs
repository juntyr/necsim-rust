use crate::{partitioning::LocalPartition, reporter::ReporterContext};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
#[allow(clippy::module_name_repetitions, clippy::too_many_arguments)]
pub trait AlmostInfiniteSimulation {
    type Error;
    type AuxiliaryArguments;

    #[debug_requires(sigma >= 0.0_f64, "standard deviation sigma must be non-negative")]
    #[debug_requires(
        (0.0_f64..=1.0_f64).contains(&speciation_probability_per_generation),
        "0.0 <= speciation_probability_per_generation <= 1.0"
    )]
    #[debug_requires(
        (0.0_f64..=1.0_f64).contains(&sample_percentage),
        "0.0 <= sample_percentage <= 1.0"
    )]
    fn simulate<R: ReporterContext, P: LocalPartition<R>>(
        radius: u32,
        sigma: f64,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        local_partition: &mut P,
        auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error>;
}
