use alloc::vec::{Drain, Vec};

use necsim_core::{
    cogs::{
        CoalescenceRngSample, CoherentLineageStore, DispersalSampler, EmigrationExit, Habitat,
        LineageReference, RngCore, SpeciationProbability,
    },
    landscape::{IndexedLocation, Location},
    lineage::MigratingLineage,
    simulation::partial::emigration_exit::PartialSimulation,
};

use crate::decomposition::Decomposition;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct DomainEmigrationExit<C: Decomposition> {
    decomposition: C,
    emigrants: Vec<(u32, MigratingLineage)>,
}

#[contract_trait]
impl<
        C: Decomposition,
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
    > EmigrationExit<H, G, N, D, R, S> for DomainEmigrationExit<C>
{
    #[must_use]
    #[inline]
    #[debug_ensures(ret.is_some() == (
        self.decomposition.map_location_to_subdomain_rank(&old(dispersal_target.clone())) ==
        self.decomposition.get_subdomain_rank()
    ), "lineage only emigrates to other subdomains")]
    fn optionally_emigrate(
        &mut self,
        lineage_reference: R,
        dispersal_origin: IndexedLocation,
        dispersal_target: Location,
        event_time: f64,
        simulation: &mut PartialSimulation<H, G, N, D, R, S>,
        rng: &mut G,
    ) -> Option<(R, IndexedLocation, Location, f64)> {
        let target_subdomain = self
            .decomposition
            .map_location_to_subdomain_rank(&dispersal_target);

        if target_subdomain == self.decomposition.get_subdomain_rank() {
            return Some((
                lineage_reference,
                dispersal_origin,
                dispersal_target,
                event_time,
            ));
        }

        self.emigrants.push((
            target_subdomain,
            MigratingLineage {
                global_reference: simulation.lineage_store.emigrate(lineage_reference),
                dispersal_origin,
                dispersal_target,
                event_time,
                coalescence_rng_sample: CoalescenceRngSample::new(rng),
            },
        ));

        None
    }
}

impl<C: Decomposition> DomainEmigrationExit<C> {
    #[must_use]
    pub fn new(decomposition: C) -> Self {
        Self {
            decomposition,
            emigrants: Vec::new(),
        }
    }

    pub fn drain_emigrants(&mut self) -> Drain<(u32, MigratingLineage)> {
        self.emigrants.drain(..)
    }
}
