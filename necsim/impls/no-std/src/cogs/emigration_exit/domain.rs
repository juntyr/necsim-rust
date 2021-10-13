use alloc::vec::Vec;
use core::marker::PhantomData;

use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_core::{
    cogs::{
        coalescence_sampler::CoalescenceRngSample, Backup, EmigrationExit, F64Core, Habitat,
        LineageReference, LocallyCoherentLineageStore, RngCore,
    },
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, MigratingLineage},
    simulation::partial::emigration_exit::PartialSimulation,
};

use crate::decomposition::Decomposition;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct DomainEmigrationExit<F: F64Core, H: Habitat<F>, C: Decomposition<F, H>> {
    decomposition: C,
    emigrants: Vec<(u32, MigratingLineage)>,
    _marker: PhantomData<(F, H)>,
}

#[contract_trait]
impl<F: F64Core, H: Habitat<F>, C: Decomposition<F, H>> Backup for DomainEmigrationExit<F, H, C> {
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
            _marker: PhantomData::<(F, H)>,
        }
    }
}

#[contract_trait]
impl<
        F: F64Core,
        H: Habitat<F>,
        C: Decomposition<F, H>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: LocallyCoherentLineageStore<F, H, R>,
    > EmigrationExit<F, H, G, R, S> for DomainEmigrationExit<F, H, C>
{
    #[must_use]
    #[debug_ensures(ret.is_some() == (
        old(self.decomposition.map_location_to_subdomain_rank(
            &dispersal_target, &simulation.habitat
        )) == self.decomposition.get_subdomain_rank()
    ), "lineage only emigrates to other subdomains")]
    fn optionally_emigrate(
        &mut self,
        global_reference: GlobalLineageReference,
        dispersal_origin: IndexedLocation,
        dispersal_target: Location,
        prior_time: NonNegativeF64,
        event_time: PositiveF64,
        simulation: &mut PartialSimulation<F, H, G, R, S>,
        rng: &mut G,
    ) -> Option<(
        GlobalLineageReference,
        IndexedLocation,
        Location,
        NonNegativeF64,
        PositiveF64,
    )> {
        let target_subdomain = self
            .decomposition
            .map_location_to_subdomain_rank(&dispersal_target, &simulation.habitat);

        if target_subdomain == self.decomposition.get_subdomain_rank() {
            return Some((
                global_reference,
                dispersal_origin,
                dispersal_target,
                prior_time,
                event_time,
            ));
        }

        self.emigrants.push((
            target_subdomain,
            MigratingLineage {
                global_reference,
                dispersal_origin,
                dispersal_target,
                prior_time,
                event_time,
                coalescence_rng_sample: CoalescenceRngSample::new(rng),
            },
        ));

        None
    }
}

impl<F: F64Core, H: Habitat<F>, C: Decomposition<F, H>> DomainEmigrationExit<F, H, C> {
    #[must_use]
    pub fn new(decomposition: C) -> Self {
        Self {
            decomposition,
            emigrants: Vec::new(),
            _marker: PhantomData::<(F, H)>,
        }
    }

    pub fn len(&self) -> usize {
        self.emigrants.len()
    }

    pub fn is_empty(&self) -> bool {
        self.emigrants.is_empty()
    }
}

impl<F: F64Core, H: Habitat<F>, C: Decomposition<F, H>> Iterator for DomainEmigrationExit<F, H, C> {
    type Item = (u32, MigratingLineage);

    fn next(&mut self) -> Option<Self::Item> {
        self.emigrants.pop()
    }
}
