use necsim_core::{
    cogs::{Backup, Habitat},
    landscape::IndexedLocation,
};

use super::EmigrationChoice;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct AlwaysEmigrationChoice(());

impl Default for AlwaysEmigrationChoice {
    fn default() -> Self {
        Self(())
    }
}

#[contract_trait]
impl Backup for AlwaysEmigrationChoice {
    unsafe fn backup_unchecked(&self) -> Self {
        Self(())
    }
}

#[contract_trait]
impl<H: Habitat> EmigrationChoice<H> for AlwaysEmigrationChoice {
    fn should_lineage_emigrate(
        &self,
        _indexed_location: &IndexedLocation,
        _time: f64,
        _habitat: &H,
    ) -> bool {
        true
    }
}
