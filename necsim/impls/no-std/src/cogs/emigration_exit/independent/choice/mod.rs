use necsim_core::{
    cogs::{Backup, Habitat, MathsCore},
    landscape::IndexedLocation,
};
use necsim_core_bond::PositiveF64;

pub mod always;
pub mod probabilistic;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait EmigrationChoice<M: MathsCore, H: Habitat<M>>: Backup + core::fmt::Debug {
    fn should_lineage_emigrate(
        &self,
        indexed_location: &IndexedLocation,
        time: PositiveF64,
        habitat: &H,
    ) -> bool;
}
