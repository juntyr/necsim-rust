use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        CoalescenceRngSample, DispersalSampler, EmigrationExit, Habitat, RngCore,
        SpeciationProbability,
    },
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, MigratingLineage},
    simulation::partial::emigration_exit::PartialSimulation,
};

use crate::{
    cogs::lineage_store::independent::IndependentLineageStore, decomposition::Decomposition,
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct IndependentEmigrationExit<H: Habitat, C: Decomposition<H>> {
    decomposition: C,
    emigrant: Option<(u32, MigratingLineage)>,
    _marker: PhantomData<H>,
}

#[contract_trait]
impl<
        H: Habitat,
        C: Decomposition<H>,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
    > EmigrationExit<H, G, N, D, GlobalLineageReference, IndependentLineageStore<H>>
    for IndependentEmigrationExit<H, C>
{
    #[must_use]
    #[inline]
    #[debug_requires(self.emigrant.is_none(), "can only hold one emigrant")]
    #[debug_ensures(ret.is_some() == (
        old(self.decomposition.map_location_to_subdomain_rank(
            &dispersal_target, &simulation.habitat
        )) == self.decomposition.get_subdomain_rank()
    ), "lineage only emigrates to other subdomains")]
    fn optionally_emigrate(
        &mut self,
        lineage_reference: GlobalLineageReference,
        dispersal_origin: IndexedLocation,
        dispersal_target: Location,
        event_time: f64,
        simulation: &mut PartialSimulation<
            H,
            G,
            N,
            D,
            GlobalLineageReference,
            IndependentLineageStore<H>,
        >,
        rng: &mut G,
    ) -> Option<(GlobalLineageReference, IndexedLocation, Location, f64)> {
        let target_subdomain = self
            .decomposition
            .map_location_to_subdomain_rank(&dispersal_target, &simulation.habitat);

        if target_subdomain == self.decomposition.get_subdomain_rank() {
            return Some((
                lineage_reference,
                dispersal_origin,
                dispersal_target,
                event_time,
            ));
        }

        self.emigrant = Some((
            target_subdomain,
            MigratingLineage {
                global_reference: lineage_reference,
                dispersal_origin,
                dispersal_target,
                event_time,
                coalescence_rng_sample: CoalescenceRngSample::new(rng),
            },
        ));

        None
    }
}

impl<H: Habitat, C: Decomposition<H>> IndependentEmigrationExit<H, C> {
    #[must_use]
    pub fn new(decomposition: C) -> Self {
        Self {
            decomposition,
            emigrant: None,
            _marker: PhantomData::<H>,
        }
    }

    pub fn len(&self) -> usize {
        self.emigrant.is_some() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.emigrant.is_none()
    }

    pub fn take(&mut self) -> Option<(u32, MigratingLineage)> {
        self.emigrant.take()
    }
}
