use std::{collections::VecDeque, marker::PhantomData};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use necsim_core::{
    cogs::{Backup, MathsCore, PrimeableRng, RngCore},
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, Lineage},
    reporter::NullReporter,
    simulation::{Simulation, SimulationBuilder},
};
use necsim_core_bond::{ClosedUnitF64, NonNegativeF64, PositiveF64};
use necsim_impls_no_std::cogs::{
    active_lineage_sampler::{
        independent::{
            event_time_sampler::poisson::PoissonEventTimeSampler, IndependentActiveLineageSampler,
        },
        singular::SingularActiveLineageSampler,
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
pub struct CorrelationSimulationRng<M: MathsCore, G: RngCore<M> + PrimeableRng<M>, const SIGMA: f64>
{
    simulation: Simulation<
        M,
        AlmostInfiniteHabitat<M>,
        InterceptingReporter<M, G>,
        GlobalLineageReference,
        IndependentLineageStore<M, AlmostInfiniteHabitat<M>>,
        NeverEmigrationExit,
        AlmostInfiniteNormalDispersalSampler<M, InterceptingReporter<M, G>>,
        IndependentCoalescenceSampler<M, AlmostInfiniteHabitat<M>>,
        UniformTurnoverRate,
        UniformSpeciationProbability,
        IndependentEventSampler<
            M,
            AlmostInfiniteHabitat<M>,
            InterceptingReporter<M, G>,
            NeverEmigrationExit,
            AlmostInfiniteNormalDispersalSampler<M, InterceptingReporter<M, G>>,
            UniformTurnoverRate,
            UniformSpeciationProbability,
        >,
        NeverImmigrationEntry,
        IndependentActiveLineageSampler<
            M,
            AlmostInfiniteHabitat<M>,
            InterceptingReporter<M, G>,
            NeverEmigrationExit,
            AlmostInfiniteNormalDispersalSampler<M, InterceptingReporter<M, G>>,
            UniformTurnoverRate,
            UniformSpeciationProbability,
            PoissonEventTimeSampler,
        >,
    >,
    other_rngs_lineages: VecDeque<(InterceptingReporter<M, G>, Lineage)>,
}

impl<M: MathsCore, G: RngCore<M, Seed: Clone> + PrimeableRng<M>, const SIGMA: f64> RngCore<M>
    for CorrelationSimulationRng<M, G, SIGMA>
{
    type Seed = G::Seed;

    fn from_seed(seed: Self::Seed) -> Self {
        let mut simulation = SimulationBuilder {
            maths: PhantomData::<M>,
            habitat: AlmostInfiniteHabitat::default(),
            lineage_reference: PhantomData::<GlobalLineageReference>,
            lineage_store: IndependentLineageStore::default(),
            dispersal_sampler: AlmostInfiniteNormalDispersalSampler::new(
                NonNegativeF64::new(SIGMA).unwrap(),
            ),
            coalescence_sampler: IndependentCoalescenceSampler::default(),
            turnover_rate: UniformTurnoverRate::default(),
            speciation_probability: UniformSpeciationProbability::new(ClosedUnitF64::zero()),
            emigration_exit: NeverEmigrationExit::default(),
            event_sampler: IndependentEventSampler::default(),
            active_lineage_sampler: IndependentActiveLineageSampler::empty(
                PoissonEventTimeSampler::new(PositiveF64::new(1.0_f64).unwrap()),
            ),
            rng: InterceptingReporter::<M, G>::from_seed(seed.clone()),
            immigration_entry: NeverImmigrationEntry::default(),
        }
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
                InterceptingReporter::<M, G>::from_seed(seed.clone()),
                Lineage::new(
                    IndexedLocation::new(Location::new(0, 1), 0),
                    simulation.habitat(),
                ),
            ),
            (
                InterceptingReporter::<M, G>::from_seed(seed.clone()),
                Lineage::new(
                    IndexedLocation::new(Location::new(1, 0), 0),
                    simulation.habitat(),
                ),
            ),
            (
                InterceptingReporter::<M, G>::from_seed(seed),
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
                .simulate_incremental_early_stop(|_, steps, _| steps >= 256, &mut NullReporter);
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
impl<M: MathsCore, G: RngCore<M> + PrimeableRng<M>, const SIGMA: f64> Backup
    for CorrelationSimulationRng<M, G, SIGMA>
{
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

impl<M: MathsCore, G: RngCore<M> + PrimeableRng<M>, const SIGMA: f64> Clone
    for CorrelationSimulationRng<M, G, SIGMA>
{
    fn clone(&self) -> Self {
        unsafe { self.backup_unchecked() }
    }
}

impl<M: MathsCore, G: RngCore<M> + PrimeableRng<M>, const SIGMA: f64> Serialize
    for CorrelationSimulationRng<M, G, SIGMA>
{
    fn serialize<S: Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
        unimplemented!()
    }
}

impl<'de, M: MathsCore, G: RngCore<M> + PrimeableRng<M>, const SIGMA: f64> Deserialize<'de>
    for CorrelationSimulationRng<M, G, SIGMA>
{
    fn deserialize<D: Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        unimplemented!()
    }
}
