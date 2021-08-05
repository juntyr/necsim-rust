use necsim_core::{
    cogs::{Backup, EmigrationExit, Habitat, LineageReference, LineageStore, RngCore},
    landscape::{IndexedLocation, Location},
    simulation::partial::emigration_exit::PartialSimulation,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::RustToCudaAsRust))]
#[derive(Debug)]
pub struct NeverEmigrationExit([u8; 0]);

impl Default for NeverEmigrationExit {
    fn default() -> Self {
        Self([])
    }
}

#[contract_trait]
impl Backup for NeverEmigrationExit {
    unsafe fn backup_unchecked(&self) -> Self {
        Self([])
    }
}

#[contract_trait]
impl<H: Habitat, G: RngCore, R: LineageReference<H>, S: LineageStore<H, R>>
    EmigrationExit<H, G, R, S> for NeverEmigrationExit
{
    #[must_use]
    #[inline]
    #[debug_ensures(ret.is_some(), "lineage never emigrates")]
    fn optionally_emigrate(
        &mut self,
        lineage_reference: R,
        dispersal_origin: IndexedLocation,
        dispersal_target: Location,
        prior_time: NonNegativeF64,
        event_time: PositiveF64,
        _simulation: &mut PartialSimulation<H, G, R, S>,
        _rng: &mut G,
    ) -> Option<(R, IndexedLocation, Location, NonNegativeF64, PositiveF64)> {
        Some((
            lineage_reference,
            dispersal_origin,
            dispersal_target,
            prior_time,
            event_time,
        ))
    }
}
