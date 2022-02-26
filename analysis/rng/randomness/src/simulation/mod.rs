use std::{marker::PhantomData, num::NonZeroU32, ops::ControlFlow};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use necsim_core::{
    cogs::{Backup, MathsCore, PrimeableRng, RngCore},
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, Lineage},
    reporter::NullReporter,
    simulation::{Simulation, SimulationBuilder},
};
use necsim_core_bond::{ClosedUnitF64, OffByOneU32, PositiveF64};
use necsim_impls_no_std::cogs::{
    active_lineage_sampler::{
        independent::{
            event_time_sampler::poisson::PoissonEventTimeSampler, IndependentActiveLineageSampler,
        },
        singular::SingularActiveLineageSampler,
    },
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
    emigration_exit::never::NeverEmigrationExit,
    event_sampler::independent::IndependentEventSampler,
    habitat::non_spatial::NonSpatialHabitat,
    immigration_entry::never::NeverImmigrationEntry,
    lineage_store::independent::IndependentLineageStore,
    origin_sampler::{non_spatial::NonSpatialOriginSampler, pre_sampler::OriginPreSampler},
    speciation_probability::uniform::UniformSpeciationProbability,
    turnover_rate::uniform::UniformTurnoverRate,
};

mod rng;
use rng::InterceptingReporter;

#[derive(Debug)]
#[allow(clippy::module_name_repetitions, clippy::type_complexity)]
pub struct SimulationRng<M: MathsCore, G: RngCore<M> + PrimeableRng<M>, const SIZE: u32> {
    simulation: Simulation<
        M,
        NonSpatialHabitat<M>,
        InterceptingReporter<M, G>,
        GlobalLineageReference,
        IndependentLineageStore<M, NonSpatialHabitat<M>>,
        NeverEmigrationExit,
        NonSpatialDispersalSampler<M, InterceptingReporter<M, G>>,
        IndependentCoalescenceSampler<M, NonSpatialHabitat<M>>,
        UniformTurnoverRate,
        UniformSpeciationProbability,
        IndependentEventSampler<
            M,
            NonSpatialHabitat<M>,
            InterceptingReporter<M, G>,
            NeverEmigrationExit,
            NonSpatialDispersalSampler<M, InterceptingReporter<M, G>>,
            UniformTurnoverRate,
            UniformSpeciationProbability,
        >,
        NeverImmigrationEntry,
        IndependentActiveLineageSampler<
            M,
            NonSpatialHabitat<M>,
            InterceptingReporter<M, G>,
            NeverEmigrationExit,
            NonSpatialDispersalSampler<M, InterceptingReporter<M, G>>,
            UniformTurnoverRate,
            UniformSpeciationProbability,
            PoissonEventTimeSampler,
        >,
    >,
}

impl<M: MathsCore, G: RngCore<M> + PrimeableRng<M>, const SIZE: u32> RngCore<M>
    for SimulationRng<M, G, SIZE>
{
    type Seed = G::Seed;

    fn from_seed(seed: Self::Seed) -> Self {
        let size = OffByOneU32::new(u64::from(SIZE)).unwrap();

        let habitat = NonSpatialHabitat::new((size, size), NonZeroU32::new(SIZE).unwrap());

        let (lineage_store, active_lineage_sampler, _) =
            IndependentActiveLineageSampler::init_with_store_and_lineages(
                NonSpatialOriginSampler::new(OriginPreSampler::none(), &habitat),
                PoissonEventTimeSampler::new(PositiveF64::new(1.0_f64).unwrap()),
            );

        let mut simulation = SimulationBuilder {
            maths: PhantomData::<M>,
            habitat,
            lineage_reference: PhantomData::<GlobalLineageReference>,
            lineage_store,
            dispersal_sampler: NonSpatialDispersalSampler::default(),
            coalescence_sampler: IndependentCoalescenceSampler::default(),
            turnover_rate: UniformTurnoverRate::default(),
            speciation_probability: UniformSpeciationProbability::new(ClosedUnitF64::zero()),
            emigration_exit: NeverEmigrationExit::default(),
            event_sampler: IndependentEventSampler::default(),
            active_lineage_sampler,
            rng: InterceptingReporter::<M, G>::from_seed(seed),
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

        Self { simulation }
    }

    fn sample_u64(&mut self) -> u64 {
        loop {
            if let Some(sample) = self.simulation.rng_mut().buffer().pop_front() {
                return sample;
            }

            self.simulation.simulate_incremental_early_stop(
                |_, steps, _| {
                    if steps >= 256 {
                        ControlFlow::BREAK
                    } else {
                        ControlFlow::CONTINUE
                    }
                },
                &mut NullReporter,
            );
        }
    }
}

#[contract_trait]
impl<M: MathsCore, G: RngCore<M> + PrimeableRng<M>, const SIZE: u32> Backup
    for SimulationRng<M, G, SIZE>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            simulation: self.simulation.backup_unchecked(),
        }
    }
}

impl<M: MathsCore, G: RngCore<M> + PrimeableRng<M>, const SIZE: u32> Clone
    for SimulationRng<M, G, SIZE>
{
    fn clone(&self) -> Self {
        unsafe { self.backup_unchecked() }
    }
}

impl<M: MathsCore, R: RngCore<M> + PrimeableRng<M>, const SIZE: u32> Serialize
    for SimulationRng<M, R, SIZE>
{
    fn serialize<S: Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
        unimplemented!()
    }
}

impl<'de, M: MathsCore, R: RngCore<M> + PrimeableRng<M>, const SIZE: u32> Deserialize<'de>
    for SimulationRng<M, R, SIZE>
{
    fn deserialize<D: Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        unimplemented!()
    }
}
