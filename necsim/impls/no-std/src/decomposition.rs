use core::num::NonZeroU32;

use necsim_core::landscape::Location;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Decomposition: Sized + core::fmt::Debug {
    #[debug_ensures(
        ret < self.get_number_of_subdomains().get(),
        "subdomain rank is in range [0, self.get_number_of_subdomains())"
    )]
    fn get_subdomain_rank(&self) -> u32;

    fn get_number_of_subdomains(&self) -> NonZeroU32;

    #[debug_ensures(
        ret < self.get_number_of_subdomains().get(),
        "subdomain rank is in range [0, self.get_number_of_subdomains())"
    )]
    fn map_location_to_subdomain_rank(&self, location: &Location) -> u32;
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct MonolithicDecomposition(());

impl Default for MonolithicDecomposition {
    fn default() -> Self {
        Self(())
    }
}

#[contract_trait]
impl Decomposition for MonolithicDecomposition {
    fn get_subdomain_rank(&self) -> u32 {
        0_u32
    }

    fn get_number_of_subdomains(&self) -> NonZeroU32 {
        unsafe { NonZeroU32::new_unchecked(1) }
    }

    fn map_location_to_subdomain_rank(&self, _location: &Location) -> u32 {
        0_u32
    }
}
