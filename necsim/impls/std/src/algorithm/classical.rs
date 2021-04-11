use std::marker::PhantomData;

use necsim_core::cogs::{
    EmigrationExit, Habitat, ImmigrationEntry, LineageStore, LocallyCoherentLineageStore, RngCore,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::classical::ClassicalActiveLineageSampler,
    coalescence_sampler::unconditional::UnconditionalCoalescenceSampler,
    event_sampler::unconditional::UnconditionalEventSampler,
    lineage_reference::in_memory::InMemoryLineageReference,
    lineage_store::coherent::locally::classical::ClassicalLineageStore,
    origin_sampler::pre_sampler::OriginPreSampler, turnover_rate::uniform::UniformTurnoverRate,
};

use crate::{algorithm::Algorithm, cogs::rng::std::StdRng, scenario::Scenario};

#[allow(clippy::module_name_repetitions, clippy::type_complexity)]
pub struct ClassicalAlgorithm<
    H: Habitat,
    L: LocallyCoherentLineageStore<H, InMemoryLineageReference>,
    O: Scenario<
        StdRng,
        ClassicalLineageStore<H>,
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
    turnover_rate: UniformTurnoverRate,
    speciation_probability: O::SpeciationProbability,
    event_sampler: UnconditionalEventSampler<
        H,
        StdRng,
        InMemoryLineageReference,
        L,
        X,
        O::DispersalSampler,
        UnconditionalCoalescenceSampler<H, InMemoryLineageReference, L>,
        UniformTurnoverRate,
        O::SpeciationProbability,
    >,
    active_lineage_sampler: ClassicalActiveLineageSampler<
        H,
        StdRng,
        InMemoryLineageReference,
        L,
        X,
        O::DispersalSampler,
        O::SpeciationProbability,
        I,
    >,
    marker: PhantomData<I>,
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct ClassicalArgs;

#[allow(clippy::type_complexity)]
impl<
        H: Habitat,
        O: Scenario<
            StdRng,
            ClassicalLineageStore<H>,
            Habitat = H,
            LineageReference = InMemoryLineageReference,
            TurnoverRate = UniformTurnoverRate,
        >,
        X: EmigrationExit<H, StdRng, InMemoryLineageReference, O::LineageStore>,
        I: ImmigrationEntry,
    > Algorithm<ClassicalLineageStore<H>, O, X, I>
    for ClassicalAlgorithm<H, O::LineageStore, O, X, I>
where
    O::LineageStore: LocallyCoherentLineageStore<H, InMemoryLineageReference>,
{
    type ActiveLineageSampler = ClassicalActiveLineageSampler<
        H,
        Self::Rng,
        Self::LineageReference,
        Self::LineageStore,
        X,
        O::DispersalSampler,
        O::SpeciationProbability,
        I,
    >;
    type Arguments = ClassicalArgs;
    type CoalescenceSampler =
        UnconditionalCoalescenceSampler<H, Self::LineageReference, Self::LineageStore>;
    type Error = O::Error;
    type EventSampler = UnconditionalEventSampler<
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

    fn initialise<P: Iterator<Item = u64>>(
        _args: Self::Arguments,
        seed: u64,
        scenario: O,
        pre_sampler: OriginPreSampler<P>,
        emigration_exit: X,
    ) -> Result<Self, Self::Error> {
        let rng = StdRng::seed_from_u64(seed);

        let origin_sampler = scenario.sample_habitat(pre_sampler);

        let lineage_store = Self::LineageStore::from_origin_sampler(origin_sampler);

        let (habitat, dispersal_sampler, turnover_rate, speciation_probability) = scenario.build();

        let coalescence_sampler = UnconditionalCoalescenceSampler::default();
        let event_sampler = UnconditionalEventSampler::default();
        let active_lineage_sampler = ClassicalActiveLineageSampler::new(&lineage_store);

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
