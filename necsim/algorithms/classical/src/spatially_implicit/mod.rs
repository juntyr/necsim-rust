use necsim_impls_no_std::{
    reporter::ReporterContext, simulation::spatially_implicit::SpatiallyImplicitSimulation,
};

mod dynamic;
mod r#static;

use super::ClassicalSimulation;

#[contract_trait]
impl SpatiallyImplicitSimulation for ClassicalSimulation {
    type AuxiliaryArguments = ();
    type Error = !;

    /// Simulates the classical coalescence algorithm on non-spatial
    /// local and meta `habitat`s with non-spatial `dispersal` and
    /// migration from the meta- to the local community.
    /// If `dynamic_meta` is true, the metacommunity will be dynamic.
    #[allow(clippy::too_many_arguments)]
    fn simulate<P: ReporterContext>(
        dynamic_meta: bool,
        local_area_deme: ((u32, u32), u32),
        meta_area_deme: ((u32, u32), u32),
        local_migration_probability_per_generation: f64,
        meta_speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        reporter_context: P,
        _auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        let (time, steps) = if dynamic_meta {
            dynamic::simulate_dynamic(
                local_area_deme,
                meta_area_deme,
                local_migration_probability_per_generation,
                meta_speciation_probability_per_generation,
                sample_percentage,
                seed,
                reporter_context,
            )
        } else {
            r#static::simulate_static(
                local_area_deme,
                meta_area_deme,
                local_migration_probability_per_generation,
                meta_speciation_probability_per_generation,
                sample_percentage,
                seed,
                reporter_context,
            )
        };

        Ok((time, steps))
    }
}
