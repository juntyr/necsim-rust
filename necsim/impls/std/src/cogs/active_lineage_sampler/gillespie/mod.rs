use std::{hash::BuildHasherDefault, iter::FromIterator, marker::PhantomData};

use fxhash::FxHasher32;
use keyed_priority_queue::KeyedPriorityQueue;
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_core::{
    cogs::{
        Backup, CoalescenceSampler, DispersalSampler, EmigrationExit, F64Core,
        GloballyCoherentLineageStore, Habitat, ImmigrationEntry, LineageReference, RngCore,
        SpeciationProbability, TurnoverRate,
    },
    landscape::Location,
};

use necsim_impls_no_std::cogs::event_sampler::gillespie::{
    GillespieEventSampler, GillespiePartialSimulation,
};

mod event_time;
mod sampler;

use event_time::EventTime;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::type_complexity)]
pub struct GillespieActiveLineageSampler<
    F: F64Core,
    H: Habitat<F>,
    G: RngCore<F>,
    R: LineageReference<F, H>,
    S: GloballyCoherentLineageStore<F, H, R>,
    X: EmigrationExit<F, H, G, R, S>,
    D: DispersalSampler<F, H, G>,
    C: CoalescenceSampler<F, H, R, S>,
    T: TurnoverRate<F, H>,
    N: SpeciationProbability<F, H>,
    E: GillespieEventSampler<F, H, G, R, S, X, D, C, T, N>,
    I: ImmigrationEntry<F>,
> {
    active_locations: KeyedPriorityQueue<Location, EventTime, BuildHasherDefault<FxHasher32>>,
    number_active_lineages: usize,
    last_event_time: NonNegativeF64,
    marker: PhantomData<(F, H, G, R, S, X, D, C, T, N, E, I)>,
}

impl<
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: GloballyCoherentLineageStore<F, H, R>,
        X: EmigrationExit<F, H, G, R, S>,
        D: DispersalSampler<F, H, G>,
        C: CoalescenceSampler<F, H, R, S>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
        E: GillespieEventSampler<F, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<F>,
    > GillespieActiveLineageSampler<F, H, G, R, S, X, D, C, T, N, E, I>
{
    #[must_use]
    pub fn new(
        partial_simulation: &GillespiePartialSimulation<F, H, G, R, S, D, C, T, N>,
        event_sampler: &E,
        rng: &mut G,
    ) -> Self {
        use necsim_core::cogs::RngSampler;

        let mut active_locations: Vec<(Location, EventTime)> = Vec::new();

        let mut number_active_lineages: usize = 0;

        partial_simulation
            .lineage_store
            .iter_active_locations(&partial_simulation.habitat)
            .for_each(|location| {
                let number_active_lineages_at_location = partial_simulation
                    .lineage_store
                    .get_local_lineage_references_at_location_unordered(
                        &location,
                        &partial_simulation.habitat,
                    )
                    .len();

                if number_active_lineages_at_location > 0 {
                    // All lineages were just initially inserted into the lineage store,
                    //  so all active lineages are in the lineage store
                    if let Ok(event_rate_at_location) = PositiveF64::new(
                        event_sampler
                            .get_event_rate_at_location(&location, partial_simulation)
                            .get(),
                    ) {
                        active_locations.push((
                            location,
                            EventTime::from(rng.sample_exponential(event_rate_at_location)),
                        ));

                        number_active_lineages += number_active_lineages_at_location;
                    }
                }
            });

        Self {
            active_locations: KeyedPriorityQueue::from_iter(active_locations),
            number_active_lineages,
            last_event_time: NonNegativeF64::zero(),
            marker: PhantomData::<(F, H, G, R, S, X, D, C, T, N, E, I)>,
        }
    }
}

impl<
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: GloballyCoherentLineageStore<F, H, R>,
        X: EmigrationExit<F, H, G, R, S>,
        D: DispersalSampler<F, H, G>,
        C: CoalescenceSampler<F, H, R, S>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
        E: GillespieEventSampler<F, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<F>,
    > core::fmt::Debug for GillespieActiveLineageSampler<F, H, G, R, S, X, D, C, T, N, E, I>
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct(stringify!(GillespieActiveLineageSampler))
            .field("active_locations", &"PriorityQueue")
            .field("number_active_lineages", &self.number_active_lineages)
            .finish()
    }
}

#[contract_trait]
impl<
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: GloballyCoherentLineageStore<F, H, R>,
        X: EmigrationExit<F, H, G, R, S>,
        D: DispersalSampler<F, H, G>,
        C: CoalescenceSampler<F, H, R, S>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
        E: GillespieEventSampler<F, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<F>,
    > Backup for GillespieActiveLineageSampler<F, H, G, R, S, X, D, C, T, N, E, I>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            active_locations: self.active_locations.clone(),
            number_active_lineages: self.number_active_lineages,
            last_event_time: self.last_event_time,
            marker: PhantomData::<(F, H, G, R, S, X, D, C, T, N, E, I)>,
        }
    }
}
