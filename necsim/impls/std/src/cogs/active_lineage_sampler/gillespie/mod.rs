use priority_queue::PriorityQueue;

use necsim_core::{
    cogs::{
        CoalescenceSampler, CoherentLineageStore, DispersalSampler, Habitat, LineageReference,
        RngCore,
    },
    landscape::Location,
    simulation::partial::event_sampler::PartialSimulation,
};

use necsim_impls_no_std::cogs::event_sampler::gillespie::GillespieEventSampler;

mod event_time;
mod sampler;

use event_time::EventTime;

#[allow(clippy::module_name_repetitions)]
pub struct GillespieActiveLineageSampler<
    H: Habitat,
    G: RngCore,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: CoherentLineageStore<H, R>,
    C: CoalescenceSampler<H, G, R, S>,
    E: GillespieEventSampler<H, G, D, R, S, C>,
> {
    active_locations: PriorityQueue<Location, EventTime>,
    number_active_lineages: usize,
    last_event_time: f64,
    marker: std::marker::PhantomData<(H, G, D, R, S, C, E)>,
}

impl<
        H: Habitat,
        G: RngCore,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        C: CoalescenceSampler<H, G, R, S>,
        E: GillespieEventSampler<H, G, D, R, S, C>,
    > GillespieActiveLineageSampler<H, G, D, R, S, C, E>
{
    #[must_use]
    pub fn new(
        partial_simulation: &PartialSimulation<H, G, D, R, S, C>,
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
                    .get_active_lineages_at_location(&location)
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
            marker: std::marker::PhantomData::<(H, G, D, R, S, C, E)>,
        }
    }
}

impl<
        H: Habitat,
        G: RngCore,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        C: CoalescenceSampler<H, G, R, S>,
        E: GillespieEventSampler<H, G, D, R, S, C>,
    > core::fmt::Debug for GillespieActiveLineageSampler<H, G, D, R, S, C, E>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GillespieActiveLineageSampler")
            .field("active_locations", &"PriorityQueue")
            .field("number_active_lineages", &self.number_active_lineages)
            .field("marker", &self.marker)
            .finish()
    }
}