use std::marker::PhantomData;

use necsim_core_bond::{NonNegativeF64, PositiveF64};
use priority_queue::PriorityQueue;

use necsim_core::{
    cogs::{
        Backup, CoalescenceSampler, DispersalSampler, EmigrationExit, GloballyCoherentLineageStore,
        Habitat, ImmigrationEntry, LineageReference, RngCore, SpeciationProbability, TurnoverRate,
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
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: GloballyCoherentLineageStore<H, R>,
    X: EmigrationExit<H, G, R, S>,
    D: DispersalSampler<H, G>,
    C: CoalescenceSampler<H, R, S>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
    E: GillespieEventSampler<H, G, R, S, X, D, C, T, N>,
    I: ImmigrationEntry,
> {
    active_locations: PriorityQueue<Location, EventTime>,
    number_active_lineages: usize,
    last_event_time: NonNegativeF64,
    marker: PhantomData<(H, G, R, S, X, D, C, T, N, E, I)>,
}

impl<
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: GloballyCoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
        E: GillespieEventSampler<H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry,
    > GillespieActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
{
    #[must_use]
    pub fn new(
        partial_simulation: &GillespiePartialSimulation<H, G, R, S, D, C, T, N>,
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
                    .get_active_local_lineage_references_at_location_unordered(
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
            active_locations: PriorityQueue::from(active_locations),
            number_active_lineages,
            last_event_time: NonNegativeF64::zero(),
            marker: PhantomData::<(H, G, R, S, X, D, C, T, N, E, I)>,
        }
    }
}

impl<
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: GloballyCoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
        E: GillespieEventSampler<H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry,
    > core::fmt::Debug for GillespieActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("GillespieActiveLineageSampler")
            .field("active_locations", &"PriorityQueue")
            .field("number_active_lineages", &self.number_active_lineages)
            .field("marker", &self.marker)
            .finish()
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: GloballyCoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
        E: GillespieEventSampler<H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry,
    > Backup for GillespieActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            active_locations: self.active_locations.clone(),
            number_active_lineages: self.number_active_lineages,
            last_event_time: self.last_event_time,
            marker: PhantomData::<(H, G, R, S, X, D, C, T, N, E, I)>,
        }
    }
}
