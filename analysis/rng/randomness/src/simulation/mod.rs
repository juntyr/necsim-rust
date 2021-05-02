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
    dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
    emigration_exit::never::NeverEmigrationExit,
    event_sampler::independent::IndependentEventSampler,
    habitat::non_spatial::NonSpatialHabitat,
    immigration_entry::never::NeverImmigrationEntry,
    lineage_store::independent::IndependentLineageStore,
    speciation_probability::uniform::UniformSpeciationProbability,
    turnover_rate::uniform::UniformTurnoverRate,
};

mod rng;
use rng::InterceptingReporter;

#[derive(Debug)]
#[allow(clippy::module_name_repetitions, clippy::type_complexity)]
pub struct SimulationRng<G: RngCore + PrimeableRng, const SIZE: u32> {
    simulation: Simulation<
        NonSpatialHabitat,
        InterceptingReporter<G>,
        GlobalLineageReference,
        IndependentLineageStore<NonSpatialHabitat>,
        NeverEmigrationExit,
        NonSpatialDispersalSampler<InterceptingReporter<G>>,
        IndependentCoalescenceSampler<NonSpatialHabitat>,
        UniformTurnoverRate,
        UniformSpeciationProbability,
        IndependentEventSampler<
            NonSpatialHabitat,
            InterceptingReporter<G>,
            NeverEmigrationExit,
            NonSpatialDispersalSampler<InterceptingReporter<G>>,
            UniformTurnoverRate,
            UniformSpeciationProbability,
        >,
        NeverImmigrationEntry,
        IndependentActiveLineageSampler<
            NonSpatialHabitat,
            InterceptingReporter<G>,
            NeverEmigrationExit,
            NonSpatialDispersalSampler<InterceptingReporter<G>>,
            UniformTurnoverRate,
            UniformSpeciationProbability,
            PoissonEventTimeSampler,
        >,
    >,
}

impl<G: RngCore + PrimeableRng, const SIZE: u32> RngCore for SimulationRng<G, SIZE> {
    type Seed = G::Seed;

    fn from_seed(seed: Self::Seed) -> Self {
        let mut simulation = Simulation::builder()
            .habitat(NonSpatialHabitat::new((SIZE, SIZE), SIZE))
            .rng(InterceptingReporter::<G>::from_seed(seed))
            .speciation_probability(UniformSpeciationProbability::new(0.0))
            .dispersal_sampler(NonSpatialDispersalSampler::default())
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

        Self { simulation }
    }

    fn sample_u64(&mut self) -> u64 {
        loop {
            if let Some(sample) = self.simulation.rng_mut().buffer().pop_front() {
                return sample;
            }

            self.simulation
                .simulate_incremental_early_stop(|_, steps| steps >= 256, &mut NullReporter);
        }
    }
}

#[contract_trait]
impl<G: RngCore + PrimeableRng, const SIZE: u32> Backup for SimulationRng<G, SIZE> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            simulation: self.simulation.backup_unchecked(),
        }
    }
}

impl<G: RngCore + PrimeableRng, const SIZE: u32> Clone for SimulationRng<G, SIZE> {
    fn clone(&self) -> Self {
        unsafe { self.backup_unchecked() }
    }
}
