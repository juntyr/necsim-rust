use std::marker::PhantomData;

use necsim_core::{
    cogs::{
        EmigrationExit, GloballyCoherentLineageStore, Habitat, ImmigrationEntry, LineageStore,
        RngCore,
    },
    simulation::partial::event_sampler::PartialSimulation,
};

use necsim_impls_no_std::cogs::{
    coalescence_sampler::unconditional::UnconditionalCoalescenceSampler,
    event_sampler::gillespie::unconditional::UnconditionalGillespieEventSampler,
    lineage_reference::in_memory::InMemoryLineageReference,
    lineage_store::coherent::globally::gillespie::GillespieLineageStore,
    origin_sampler::pre_sampler::OriginPreSampler,
};

use crate::{
    algorithm::Algorithm,
    cogs::{active_lineage_sampler::gillespie::GillespieActiveLineageSampler, rng::std::StdRng},
    scenario::Scenario,
};

#[allow(clippy::module_name_repetitions, clippy::type_complexity)]
pub struct GillespieAlgorithm<
    H: Habitat,
    L: GloballyCoherentLineageStore<H, InMemoryLineageReference>,
    O: Scenario<
        StdRng,
        GillespieLineageStore<H>,
        Habitat = H,
        LineageReference = InMemoryLineageReference,
        LineageStore = L,
    >,
    X: EmigrationExit<H, StdRng, InMemoryLineageReference, O::LineageStore>,
    I: ImmigrationEntry,
> {
    habitat: H,
    rng: StdRng,
    lineage_store: L,
    emigration_exit: X,
    dispersal_sampler: O::DispersalSampler,
    coalescence_sampler: UnconditionalCoalescenceSampler<H, InMemoryLineageReference, L>,
    turnover_rate: O::TurnoverRate,
    speciation_probability: O::SpeciationProbability,
    event_sampler: UnconditionalGillespieEventSampler<
        H,
        StdRng,
        InMemoryLineageReference,
        L,
        X,
        O::DispersalSampler,
        UnconditionalCoalescenceSampler<H, InMemoryLineageReference, L>,
        O::TurnoverRate,
        O::SpeciationProbability,
    >,
    active_lineage_sampler: GillespieActiveLineageSampler<
        H,
        StdRng,
        InMemoryLineageReference,
        L,
        X,
        O::DispersalSampler,
        UnconditionalCoalescenceSampler<H, InMemoryLineageReference, L>,
        O::TurnoverRate,
        O::SpeciationProbability,
        UnconditionalGillespieEventSampler<
            H,
            StdRng,
            InMemoryLineageReference,
            L,
            X,
            O::DispersalSampler,
            UnconditionalCoalescenceSampler<H, InMemoryLineageReference, L>,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        I,
    >,
    marker: PhantomData<I>,
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct GillespieArguments;

#[allow(clippy::type_complexity)]
impl<
        H: Habitat,
        O: Scenario<
            StdRng,
            GillespieLineageStore<H>,
            Habitat = H,
            LineageReference = InMemoryLineageReference,
        >,
        X: EmigrationExit<H, StdRng, InMemoryLineageReference, O::LineageStore>,
        I: ImmigrationEntry,
    > Algorithm<GillespieLineageStore<H>, O, X, I>
    for GillespieAlgorithm<H, O::LineageStore, O, X, I>
where
    O::LineageStore: GloballyCoherentLineageStore<H, InMemoryLineageReference>,
{
    type ActiveLineageSampler = GillespieActiveLineageSampler<
        H,
        Self::Rng,
        Self::LineageReference,
        Self::LineageStore,
        X,
        O::DispersalSampler,
        Self::CoalescenceSampler,
        O::TurnoverRate,
        O::SpeciationProbability,
        Self::EventSampler,
        I,
    >;
    type Arguments = GillespieArguments;
    type CoalescenceSampler =
        UnconditionalCoalescenceSampler<H, Self::LineageReference, Self::LineageStore>;
    type Error = O::Error;
    type EventSampler = UnconditionalGillespieEventSampler<
        H,
        Self::Rng,
        Self::LineageReference,
        Self::LineageStore,
        X,
        O::DispersalSampler,
        Self::CoalescenceSampler,
        O::TurnoverRate,
        O::SpeciationProbability,
    >;
    type LineageReference = InMemoryLineageReference;
    type LineageStore = O::LineageStore;
    type Rng = StdRng;

    #[allow(clippy::shadow_unrelated)]
    fn initialise<P: Iterator<Item = u64>>(
        _args: Self::Arguments,
        seed: u64,
        scenario: O,
        pre_sampler: OriginPreSampler<P>,
        emigration_exit: X,
    ) -> Result<Self, Self::Error> {
        let mut rng = StdRng::seed_from_u64(seed);

        let origin_sampler = scenario.sample_habitat(pre_sampler);

        let lineage_store = Self::LineageStore::from_origin_sampler(origin_sampler);

        let (habitat, dispersal_sampler, turnover_rate, speciation_probability) = scenario.build();

        let coalescence_sampler = UnconditionalCoalescenceSampler::default();
        let event_sampler = UnconditionalGillespieEventSampler::default();

        // Pack a PartialSimulation to initialise the GillespieActiveLineageSampler
        let partial_simulation = PartialSimulation {
            habitat,
            speciation_probability,
            dispersal_sampler,
            lineage_reference: PhantomData::<Self::LineageReference>,
            lineage_store,
            emigration_exit,
            coalescence_sampler,
            turnover_rate,
            _rng: PhantomData::<StdRng>,
        };

        let active_lineage_sampler =
            GillespieActiveLineageSampler::new(&partial_simulation, &event_sampler, &mut rng);

        // Unpack the PartialSimulation to create the full Simulation
        let PartialSimulation {
            habitat,
            speciation_probability,
            dispersal_sampler,
            lineage_reference: _,
            lineage_store,
            emigration_exit,
            coalescence_sampler,
            turnover_rate,
            _rng: _,
        } = partial_simulation;

        Ok(Self {
            habitat,
            rng,
            lineage_store,
            emigration_exit,
            dispersal_sampler,
            coalescence_sampler,
            turnover_rate,
            speciation_probability,
            event_sampler,
            active_lineage_sampler,
            marker: PhantomData,
        })
    }

    fn build(
        self,
    ) -> (
        H,
        Self::Rng,
        Self::LineageStore,
        X,
        O::DispersalSampler,
        Self::CoalescenceSampler,
        O::TurnoverRate,
        O::SpeciationProbability,
        Self::EventSampler,
        Self::ActiveLineageSampler,
    ) {
        (
            self.habitat,
            self.rng,
            self.lineage_store,
            self.emigration_exit,
            self.dispersal_sampler,
            self.coalescence_sampler,
            self.turnover_rate,
            self.speciation_probability,
            self.event_sampler,
            self.active_lineage_sampler,
        )
    }
}
