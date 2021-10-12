use core::num::NonZeroU32;

use necsim_core::{
    cogs::{Backup, Habitat, F64Core},
    landscape::Location,
};

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
impl<F: F64Core, H: Habitat<F>> Decomposition<F, H> for MonolithicDecomposition {
    fn get_subdomain_rank(&self) -> u32 {
        0_u32
    }

    fn get_number_of_subdomains(&self) -> NonZeroU32 {
        unsafe { NonZeroU32::new_unchecked(1) }
    }

    fn map_location_to_subdomain_rank(&self, _location: &Location, _habitat: &H) -> u32 {
        0_u32
    }
}
