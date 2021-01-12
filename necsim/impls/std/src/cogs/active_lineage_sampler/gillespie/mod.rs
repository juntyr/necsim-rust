use priority_queue::PriorityQueue;

use necsim_core::{
    cogs::{
        CoalescenceSampler, CoherentLineageStore, DispersalSampler, EmigrationExit, Habitat,
        ImmigrationEntry, LineageReference, RngCore, SpeciationProbability,
    },
    landscape::Location,
    simulation::partial::event_sampler::PartialSimulation,
};

use necsim_impls_no_std::cogs::event_sampler::gillespie::GillespieEventSampler;

mod event_time;
mod sampler;

use event_time::EventTime;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::type_complexity)]
pub struct GillespieActiveLineageSampler<
    H: Habitat,
    G: RngCore,
    N: SpeciationProbability<H>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: CoherentLineageStore<H, R>,
    X: EmigrationExit<H, G, N, D, R, S>,
    C: CoalescenceSampler<H, R, S>,
    E: GillespieEventSampler<H, G, N, D, R, S, X, C>,
    I: ImmigrationEntry,
> {
    active_locations: PriorityQueue<Location, EventTime>,
    number_active_lineages: usize,
    last_event_time: f64,
    marker: std::marker::PhantomData<(H, G, N, D, R, S, X, C, E, I)>,
}

impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
        C: CoalescenceSampler<H, R, S>,
        E: GillespieEventSampler<H, G, N, D, R, S, X, C>,
        I: ImmigrationEntry,
    > GillespieActiveLineageSampler<H, G, N, D, R, S, X, C, E, I>
{
    #[must_use]
    pub fn new(
        partial_simulation: &PartialSimulation<H, G, N, D, R, S, X, C>,
        event_sampler: &E,
        rng: &mut G,
    ) -> Self {
        use necsim_core::cogs::RngSampler;

        let mut active_locations: Vec<(Location, EventTime)> = Vec::new();

        let mut number_active_lineages: usize = 0;

        partial_simulation
            .lineage_store
            .iter_active_locations()
            .for_each(|location| {
                let number_active_lineages_at_location = partial_simulation
                    .lineage_store
                    .get_active_local_lineage_references_at_location_unordered(&location)
                    .len();

                if number_active_lineages_at_location > 0 {
                    let event_rate_at_location = event_sampler.get_event_rate_at_location(
                        &location,
                        partial_simulation,
                        true,
                    );

                    active_locations.push((
                        location,
                        EventTime::from(rng.sample_exponential(event_rate_at_location)),
                    ));

                    number_active_lineages += number_active_lineages_at_location;
                }
            });

        active_locations.shrink_to_fit();

        Self {
            active_locations: PriorityQueue::from(active_locations),
            number_active_lineages,
            last_event_time: 0.0_f64,
            marker: std::marker::PhantomData::<(H, G, N, D, R, S, X, C, E, I)>,
        }
    }
}

impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
        C: CoalescenceSampler<H, R, S>,
        E: GillespieEventSampler<H, G, N, D, R, S, X, C>,
        I: ImmigrationEntry,
    > core::fmt::Debug for GillespieActiveLineageSampler<H, G, N, D, R, S, X, C, E, I>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GillespieActiveLineageSampler")
            .field("active_locations", &"PriorityQueue")
            .field("number_active_lineages", &self.number_active_lineages)
            .field("marker", &self.marker)
            .finish()
    }
}
