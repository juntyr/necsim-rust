use std::collections::VecDeque;

use necsim_core::{
    cogs::{Backup, PrimeableRng, RngCore, SingularActiveLineageSampler},
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, Lineage},
    reporter::NullReporter,
    simulation::Simulation,
};
use necsim_impls_no_std::cogs::{
    active_lineage_sampler::independent::{
        event_time_sampler::poisson::PoissonEventTimeSampler, IndependentActiveLineageSampler,
    },
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    dispersal_sampler::almost_infinite_normal::AlmostInfiniteNormalDispersalSampler,
    emigration_exit::never::NeverEmigrationExit,
    event_sampler::independent::IndependentEventSampler,
    habitat::almost_infinite::AlmostInfiniteHabitat,
    immigration_entry::never::NeverImmigrationEntry,
    lineage_store::independent::IndependentLineageStore,
    speciation_probability::uniform::UniformSpeciationProbability,
    turnover_rate::uniform::UniformTurnoverRate,
};

mod rng;
use rng::InterceptingReporter;

#[derive(Debug)]
#[allow(clippy::module_name_repetitions, clippy::type_complexity)]
pub struct CorrelationSimulationRng<G: RngCore + PrimeableRng, const SIGMA: f64> {
    simulation: Simulation<
        AlmostInfiniteHabitat,
        InterceptingReporter<G>,
        GlobalLineageReference,
        IndependentLineageStore<AlmostInfiniteHabitat>,
        NeverEmigrationExit,
        AlmostInfiniteNormalDispersalSampler<InterceptingReporter<G>>,
        IndependentCoalescenceSampler<AlmostInfiniteHabitat>,
        UniformTurnoverRate,
        UniformSpeciationProbability,
        IndependentEventSampler<
            AlmostInfiniteHabitat,
            InterceptingReporter<G>,
            NeverEmigrationExit,
            AlmostInfiniteNormalDispersalSampler<InterceptingReporter<G>>,
            UniformTurnoverRate,
            UniformSpeciationProbability,
        >,
        NeverImmigrationEntry,
        IndependentActiveLineageSampler<
            AlmostInfiniteHabitat,
            InterceptingReporter<G>,
            NeverEmigrationExit,
            AlmostInfiniteNormalDispersalSampler<InterceptingReporter<G>>,
            UniformTurnoverRate,
            UniformSpeciationProbability,
            PoissonEventTimeSampler,
        >,
    >,
    other_rngs_lineages: VecDeque<(InterceptingReporter<G>, Lineage)>,
}

impl<G: RngCore<Seed: Clone> + PrimeableRng, const SIGMA: f64> RngCore
    for CorrelationSimulationRng<G, SIGMA>
{
    type Seed = G::Seed;

    fn from_seed(seed: Self::Seed) -> Self {
        let mut simulation = Simulation::builder()
            .habitat(AlmostInfiniteHabitat::default())
            .rng(InterceptingReporter::<G>::from_seed(seed.clone()))
            .speciation_probability(UniformSpeciationProbability::new(0.0))
            .dispersal_sampler(AlmostInfiniteNormalDispersalSampler::new(SIGMA))
            .lineage_reference(std::marker::PhantomData::<GlobalLineageReference>)
            .lineage_store(IndependentLineageStore::default())
            .emigration_exit(NeverEmigrationExit::default())
            .coalescence_sampler(IndependentCoalescenceSampler::default())
            .turnover_rate(UniformTurnoverRate::default())
            .event_sampler(IndependentEventSampler::default())
            .immigration_entry(NeverImmigrationEntry::default())
            .active_lineage_sampler(IndependentActiveLineageSampler::empty(
                PoissonEventTimeSampler::new(1.0),
            ))
            .build();

        let lineage = Lineage::new(
            IndexedLocation::new(Location::new(0, 0), 0),
            simulation.habitat(),
        );

        let _ = simulation
            .active_lineage_sampler_mut()
            .replace_active_lineage(Some(lineage));

        let other_rngs_lineages = vec![
            (
                InterceptingReporter::<G>::from_seed(seed.clone()),
                Lineage::new(
                    IndexedLocation::new(Location::new(0, 1), 0),
                    simulation.habitat(),
                ),
            ),
            (
                InterceptingReporter::<G>::from_seed(seed.clone()),
                Lineage::new(
                    IndexedLocation::new(Location::new(1, 0), 0),
                    simulation.habitat(),
                ),
            ),
            (
                InterceptingReporter::<G>::from_seed(seed),
                Lineage::new(
                    IndexedLocation::new(Location::new(1, 1), 0),
                    simulation.habitat(),
                ),
            ),
        ]
        .into();

        Self {
            simulation,
            other_rngs_lineages,
        }
    }

    fn sample_u64(&mut self) -> u64 {
        let sample = loop {
            if let Some(sample) = self.simulation.rng_mut().buffer().pop_front() {
                break sample;
            }

            self.simulation
                .simulate_incremental_early_stop(|_, steps| steps >= 256, &mut NullReporter);
        };

        let (mut next_rng, next_lineage) = self.other_rngs_lineages.pop_front().unwrap();

        std::mem::swap(self.simulation.rng_mut(), &mut next_rng);
        let prev_rng = next_rng;

        let prev_lineage = self
            .simulation
            .active_lineage_sampler_mut()
            .replace_active_lineage(Some(next_lineage))
            .unwrap();

        self.other_rngs_lineages.push_back((prev_rng, prev_lineage));

        sample
    }
}

#[contract_trait]
impl<G: RngCore + PrimeableRng, const SIGMA: f64> Backup for CorrelationSimulationRng<G, SIGMA> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            simulation: self.simulation.backup_unchecked(),
            other_rngs_lineages: self
                .other_rngs_lineages
                .iter()
                .map(|(rng, lineage)| (rng.clone(), lineage.clone()))
                .collect(),
        }
    }
}

impl<G: RngCore + PrimeableRng, const SIGMA: f64> Clone for CorrelationSimulationRng<G, SIGMA> {
    fn clone(&self) -> Self {
        unsafe { self.backup_unchecked() }
    }
}
