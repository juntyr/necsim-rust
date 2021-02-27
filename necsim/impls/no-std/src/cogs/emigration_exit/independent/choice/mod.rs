use necsim_core::{
    cogs::{Backup, Habitat},
    landscape::IndexedLocation,
};

pub mod always;
pub mod probabilistic;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait EmigrationChoice<H: Habitat>: Backup + core::fmt::Debug {
    #[debug_requires(time >= 0.0_f64, "event times must be non-negative")]
    fn should_lineage_emigrate(
        &self,
        indexed_location: &IndexedLocation,
        time: f64,
        habitat: &H,
    ) -> bool;
}
