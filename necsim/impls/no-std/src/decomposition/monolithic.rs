use necsim_core::{
    cogs::{Backup, Habitat, MathsCore},
    landscape::Location,
};
use necsim_core_bond::Partition;

use crate::decomposition::Decomposition;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Default)]
pub struct MonolithicDecomposition(());

#[contract_trait]
impl Backup for MonolithicDecomposition {
    unsafe fn backup_unchecked(&self) -> Self {
        Self(())
    }
}

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>> Decomposition<M, H> for MonolithicDecomposition {
    fn get_subdomain(&self) -> Partition {
        Partition::monolithic()
    }

    fn map_location_to_subdomain_rank(&self, _location: &Location, _habitat: &H) -> u32 {
        0_u32
    }
}
