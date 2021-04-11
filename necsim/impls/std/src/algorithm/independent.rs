use std::num::{NonZeroU64, NonZeroUsize};

use necsim_core::{
    cogs::{EmigrationExit, Habitat, RngCore},
    lineage::GlobalLineageReference,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::independent::{
        event_time_sampler::exp::ExpEventTimeSampler, IndependentActiveLineageSampler,
    },
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    event_sampler::independent::IndependentEventSampler,
    immigration_entry::never::NeverImmigrationEntry,
    lineage_reference::in_memory::InMemoryLineageReference,
    lineage_store::{
        coherent::locally::classical::ClassicalLineageStore, independent::IndependentLineageStore,
    },
    origin_sampler::pre_sampler::OriginPreSampler,
    rng::seahash::SeaHash,
};

use crate::{
    algorithm::Algorithm,
    bounded::{Partition, PositiveF64},
    scenario::Scenario,
};

#[allow(clippy::module_name_repetitions, clippy::type_complexity)]
pub struct IndependentAlgorithm<
    H: Habitat,
    O: Scenario<
        SeaHash,
        ClassicalLineageStore<H>, // Meaningless
        Habitat = H,
        LineageReference = InMemoryLineageReference, // Meaningless
    >,
    X: EmigrationExit<H, SeaHash, GlobalLineageReference, IndependentLineageStore<H>>,
> {
    habitat: H,
    rng: SeaHash,
    lineage_store: IndependentLineageStore<H>,
    emigration_exit: X,
    dispersal_sampler: O::DispersalSampler,
    coalescence_sampler: IndependentCoalescenceSampler<H>,
    turnover_rate: O::TurnoverRate,
    speciation_probability: O::SpeciationProbability,
    event_sampler: IndependentEventSampler<
        H,
        SeaHash,
        X,
        O::DispersalSampler,
        O::TurnoverRate,
        O::SpeciationProbability,
    >,
    active_lineage_sampler: IndependentActiveLineageSampler<
        H,
        SeaHash,
        X,
        O::DispersalSampler,
        O::TurnoverRate,
        O::SpeciationProbability,
        ExpEventTimeSampler,
    >,
}

#[derive(Debug)]
pub struct AbsoluteDedupCache {
    capacity: NonZeroUsize,
}

#[derive(Debug)]
pub struct RelativeDedupCache {
    factor: PositiveF64,
}

#[derive(Debug)]
pub enum DedupCache {
    Absolute(AbsoluteDedupCache),
    Relative(RelativeDedupCache),
    None,
}

#[derive(Debug)]
pub enum PartitionMode {
    Monolithic,
    Individuals,
    IsolatedIndividuals(Partition),
    Landscape,
    Probabilistic,
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct IndependentArguments {
    delta_t: PositiveF64,
    step_slice: NonZeroU64,
    dedup_cache: DedupCache,
    partition_mode: PartitionMode,
}

#[allow(clippy::type_complexity)]
impl<
        H: Habitat,
        O: Scenario<
            SeaHash,
            ClassicalLineageStore<H>, // Meaningless
            Habitat = H,
            LineageReference = InMemoryLineageReference, // Meaningless
        >,
        X: EmigrationExit<H, SeaHash, GlobalLineageReference, IndependentLineageStore<H>>,
    > Algorithm<ClassicalLineageStore<H>, O, X, NeverImmigrationEntry>
    for IndependentAlgorithm<H, O, X>
{
    type ActiveLineageSampler = IndependentActiveLineageSampler<
        H,
        Self::Rng,
        X,
        O::DispersalSampler,
        O::TurnoverRate,
        O::SpeciationProbability,
        ExpEventTimeSampler,
    >;
    type Arguments = IndependentArguments;
    type CoalescenceSampler = IndependentCoalescenceSampler<H>;
    type Error = O::Error;
    type EventSampler = IndependentEventSampler<
        H,
        Self::Rng,
        X,
        O::DispersalSampler,
        O::TurnoverRate,
        O::SpeciationProbability,
    >;
    type LineageReference = GlobalLineageReference;
    type LineageStore = IndependentLineageStore<H>;
    type Rng = SeaHash;

    fn initialise<P: Iterator<Item = u64>>(
        args: Self::Arguments,
        seed: u64,
        scenario: O,
        _pre_sampler: OriginPreSampler<P>,
        emigration_exit: X,
    ) -> Result<Self, Self::Error> {
        let rng = SeaHash::seed_from_u64(seed);

        // let origin_sampler = scenario.sample_habitat(pre_sampler);

        let lineage_store = IndependentLineageStore::default();

        let (habitat, dispersal_sampler, turnover_rate, speciation_probability) = scenario.build();

        let coalescence_sampler = IndependentCoalescenceSampler::default();
        let event_sampler = IndependentEventSampler::default();
        let active_lineage_sampler =
            IndependentActiveLineageSampler::empty(ExpEventTimeSampler::new(args.delta_t.get()));

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
