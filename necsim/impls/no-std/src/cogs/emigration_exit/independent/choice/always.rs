use necsim_core::{
    cogs::{Backup, F64Core, Habitat},
    landscape::IndexedLocation,
};
use necsim_core_bond::PositiveF64;

use super::EmigrationChoice;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default)]
pub struct AlwaysEmigrationChoice([u8; 0]);

#[contract_trait]
impl Backup for AlwaysEmigrationChoice {
    unsafe fn backup_unchecked(&self) -> Self {
        Self([])
    }
}

#[contract_trait]
impl<F: F64Core, H: Habitat<F>> EmigrationChoice<F, H> for AlwaysEmigrationChoice {
    fn should_lineage_emigrate(
        &self,
        _indexed_location: &IndexedLocation,
        _time: PositiveF64,
        _habitat: &H,
    ) -> bool {
        true
    }
}
