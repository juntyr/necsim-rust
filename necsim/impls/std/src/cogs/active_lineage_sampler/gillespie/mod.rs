use std::{hash::BuildHasherDefault, iter::FromIterator, marker::PhantomData};

use fxhash::FxHasher32;
use keyed_priority_queue::KeyedPriorityQueue;
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_core::{
    cogs::{
        Backup, CoalescenceSampler, DispersalSampler, EmigrationExit, GloballyCoherentLineageStore,
        Habitat, ImmigrationEntry, LineageReference, MathsCore, RngCore, SpeciationProbability,
        TurnoverRate,
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
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    R: LineageReference<M, H>,
    S: GloballyCoherentLineageStore<M, H, R>,
    X: EmigrationExit<M, H, G, R, S>,
    D: DispersalSampler<M, H, G>,
    C: CoalescenceSampler<M, H, R, S>,
    T: TurnoverRate<M, H>,
    N: SpeciationProbability<M, H>,
    E: GillespieEventSampler<M, H, G, R, S, X, D, C, T, N>,
    I: ImmigrationEntry<M>,
> {
    active_locations: KeyedPriorityQueue<Location, EventTime, BuildHasherDefault<FxHasher32>>,
    number_active_lineages: usize,
    last_event_time: NonNegativeF64,
    marker: PhantomData<(M, H, G, R, S, X, D, C, T, N, E, I)>,
}

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: GloballyCoherentLineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: GillespieEventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
    > GillespieActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
{
    #[must_use]
    pub fn new(
        partial_simulation: &GillespiePartialSimulation<M, H, G, R, S, D, C, T, N>,
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
            marker: PhantomData::<(M, H, G, R, S, X, D, C, T, N, E, I)>,
        }
    }
}

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: GloballyCoherentLineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: GillespieEventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
    > core::fmt::Debug for GillespieActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
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
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: GloballyCoherentLineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: GillespieEventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
    > Backup for GillespieActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            active_locations: self.active_locations.clone(),
            number_active_lineages: self.number_active_lineages,
            last_event_time: self.last_event_time,
            marker: PhantomData::<(M, H, G, R, S, X, D, C, T, N, E, I)>,
        }
    }
}
