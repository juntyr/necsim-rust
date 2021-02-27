use alloc::vec::Vec;
use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        Backup, CoalescenceRngSample, CoherentLineageStore, DispersalSampler, EmigrationExit,
        Habitat, LineageReference, RngCore, SpeciationProbability,
    },
    landscape::{IndexedLocation, Location},
    lineage::MigratingLineage,
    simulation::partial::emigration_exit::PartialSimulation,
};

use crate::decomposition::Decomposition;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct DomainEmigrationExit<H: Habitat, C: Decomposition<H>> {
    decomposition: C,
    emigrants: Vec<(u32, MigratingLineage)>,
    _marker: PhantomData<H>,
}

#[contract_trait]
impl<H: Habitat, C: Decomposition<H>> Backup for DomainEmigrationExit<H, C> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            decomposition: self.decomposition.backup_unchecked(),
            emigrants: self
                .emigrants
                .iter()
                .map(|(partition, migrating_lineage)| {
                    (*partition, migrating_lineage.backup_unchecked())
                })
                .collect(),
            _marker: PhantomData::<H>,
        }
    }
}

#[contract_trait]
impl<
        H: Habitat,
        C: Decomposition<H>,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
    > EmigrationExit<H, G, N, D, R, S> for DomainEmigrationExit<H, C>
{
    #[must_use]
    #[debug_ensures(ret.is_some() == (
        old(self.decomposition.map_location_to_subdomain_rank(
            &dispersal_target, &simulation.habitat
        )) == self.decomposition.get_subdomain_rank()
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
            .map_location_to_subdomain_rank(&dispersal_target, &simulation.habitat);

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

impl<H: Habitat, C: Decomposition<H>> DomainEmigrationExit<H, C> {
    #[must_use]
    pub fn new(decomposition: C) -> Self {
        Self {
            decomposition,
            emigrants: Vec::new(),
            _marker: PhantomData::<H>,
        }
    }

    pub fn len(&self) -> usize {
        self.emigrants.len()
    }

    pub fn is_empty(&self) -> bool {
        self.emigrants.is_empty()
    }
}

impl<H: Habitat, C: Decomposition<H>> Iterator for DomainEmigrationExit<H, C> {
    type Item = (u32, MigratingLineage);

    fn next(&mut self) -> Option<Self::Item> {
        self.emigrants.pop()
    }
}
