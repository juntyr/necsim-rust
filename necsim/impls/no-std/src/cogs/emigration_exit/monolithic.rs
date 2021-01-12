use necsim_core::{
    cogs::{
        DispersalSampler, EmigrationExit, Habitat, LineageReference, LineageStore, RngCore,
        SpeciationProbability,
    },
    landscape::{IndexedLocation, Location},
    simulation::partial::migration::PartialSimulation,
};

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[derive(Debug)]
pub struct MonolithicEmigrationExit(());

#[contract_trait]
impl<
        // TODO: Can we assert that the habitat must be monolithic here?
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
    > EmigrationExit<H, G, N, D, R, S> for MonolithicEmigrationExit
{
    #[must_use]
    #[inline]
    #[debug_ensures(ret.is_some(), "lineage never emigrates")]
    fn optionally_emigrate(
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
