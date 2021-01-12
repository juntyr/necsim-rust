use necsim_core::{
    cogs::{
        DispersalSampler, Habitat, ImmigrationEntry, LineageReference, LineageStore, RngCore,
        SpeciationProbability,
    },
    simulation::partial::migration::PartialSimulation,
};

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[derive(Debug)]
pub struct MonolithicImmigrationEntry(());

impl Default for MonolithicImmigrationEntry {
    fn default() -> Self {
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
    > ImmigrationEntry<H, G, N, D, R, S> for MonolithicImmigrationEntry
{
    #[must_use]
    #[inline]
    fn next_optional_immigration(
        &mut self,
        _next_event_time: f64,
        _simulation: &mut PartialSimulation<H, G, N, D, R, S>,
        _rng: &mut G,
    ) -> Option<R> {
        None
    }
}
