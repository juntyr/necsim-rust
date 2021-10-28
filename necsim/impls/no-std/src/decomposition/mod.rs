use necsim_core::{
    cogs::{Backup, Habitat, MathsCore},
    landscape::Location,
};
use necsim_core_bond::Partition;

pub mod equal;
pub mod modulo;
pub mod monolithic;
pub mod radial;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Decomposition<M: MathsCore, H: Habitat<M>>: Backup + Sized + core::fmt::Debug {
    fn get_subdomain(&self) -> Partition;

    #[debug_requires(
        habitat.contains(location),
        "location is contained inside habitat"
    )]
    #[debug_ensures(
        ret < self.get_subdomain().size().get(),
        "subdomain rank is in range [0, self.get_subdomain().size())"
    )]
    fn map_location_to_subdomain_rank(&self, location: &Location, habitat: &H) -> u32;
}
