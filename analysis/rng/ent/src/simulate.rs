use std::{
    collections::VecDeque,
    num::{NonZeroU32, NonZeroU64},
};

use necsim_core::{
    cogs::PrimeableRng,
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, Lineage},
    reporter::{FilteredReporter, NullReporter},
    simulation::Simulation,
};
use necsim_impls_no_std::{
    cogs::{
        active_lineage_sampler::independent::{
            event_time_sampler::exp::ExpEventTimeSampler, IndependentActiveLineageSampler,
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
    },
    parallelisation,
    partitioning::monolithic::live::LiveMonolithicLocalPartition,
};
use parallelisation::independent::DedupCache;

pub fn simulate<R: PrimeableRng<NonSpatialHabitat>>(rng: R, speciation_probability: f64) {
    let habitat = NonSpatialHabitat::new((100, 100), 100);
    let dispersal_sampler = NonSpatialDispersalSampler::default();
    let turnover_rate = UniformTurnoverRate::default();
    let speciation_probability = UniformSpeciationProbability::new(speciation_probability);
    let lineage_store = IndependentLineageStore::default();
    let coalescence_sampler = IndependentCoalescenceSampler::default();
    let event_sampler = IndependentEventSampler::default();
    let emigration_exit = NeverEmigrationExit::default();
    let immigration_entry = NeverImmigrationEntry::default();
    let active_lineage_sampler =
        IndependentActiveLineageSampler::empty(ExpEventTimeSampler::new(1.0));

    let lineage = Lineage::new(IndexedLocation::new(Location::new(0, 0), 0), &habitat);

    let lineages = VecDeque::from(vec![lineage]);

    let simulation = Simulation::builder()
        .habitat(habitat)
        .rng(rng)
        .speciation_probability(speciation_probability)
        .dispersal_sampler(dispersal_sampler)
        .lineage_reference(std::marker::PhantomData::<GlobalLineageReference>)
        .lineage_store(lineage_store)
        .emigration_exit(emigration_exit)
        .coalescence_sampler(coalescence_sampler)
        .turnover_rate(turnover_rate)
        .event_sampler(event_sampler)
        .immigration_entry(immigration_entry)
        .active_lineage_sampler(active_lineage_sampler)
        .build();

    let reporter = FilteredReporter::from(NullReporter);
    let mut local_partition = LiveMonolithicLocalPartition::from_reporter(reporter);

    parallelisation::independent::monolithic::simulate(
        simulation,
        lineages,
        DedupCache::None,
        NonZeroU64::new(1).unwrap(),
        NonZeroU32::new(u32::MAX).unwrap(),
        &mut local_partition,
    );
}
