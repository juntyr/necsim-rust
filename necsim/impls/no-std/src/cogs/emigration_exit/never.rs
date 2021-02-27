use necsim_core::{
    cogs::{
        Backup, DispersalSampler, EmigrationExit, Habitat, LineageReference, LineageStore, RngCore,
        SpeciationProbability,
    },
    landscape::{IndexedLocation, Location},
    simulation::partial::emigration_exit::PartialSimulation,
};

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[derive(Debug)]
pub struct NeverEmigrationExit(());

impl Default for NeverEmigrationExit {
    fn default() -> Self {
        Self(())
    }
}

#[contract_trait]
impl Backup for NeverEmigrationExit {
    unsafe fn backup_unchecked(&self) -> Self {
        Self(())
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
    > EmigrationExit<H, G, N, D, R, S> for NeverEmigrationExit
{
    #[must_use]
    #[inline]
    #[debug_ensures(ret.is_some(), "lineage never emigrates")]
    fn optionally_emigrate(
        &mut self,
        lineage_reference: R,
        dispersal_origin: IndexedLocation,
        dispersal_target: Location,
        event_time: f64,
        _simulation: &mut PartialSimulation<H, G, N, D, R, S>,
        _rng: &mut G,
    ) -> Option<(R, IndexedLocation, Location, f64)> {
        Some((
            lineage_reference,
            dispersal_origin,
            dispersal_target,
            event_time,
        ))
    }
}
