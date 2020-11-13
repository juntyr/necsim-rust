use std::marker::PhantomData;

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
        speciation_probability_per_generation: f64,
        habitat: &H,
        dispersal_sampler: &D,
        lineage_store: &S,
        coalescence_sampler: &C,
        event_sampler: &E,
        rng: &mut G,
    ) -> Self {
        use necsim_core::cogs::RngSampler;

        let landscape_extent = habitat.get_extent();

        let mut active_locations: Vec<(Location, EventTime)> = Vec::with_capacity(
            landscape_extent.width() as usize * landscape_extent.height() as usize,
        );

        let lineage_reference = std::marker::PhantomData::<R>;
        let partial_simulation = PartialSimulation {
            speciation_probability_per_generation: &speciation_probability_per_generation,
            rng: PhantomData::<G>,
            habitat,
            dispersal_sampler,
            lineage_reference: &lineage_reference,
            lineage_store,
            coalescence_sampler,
        };

        let mut number_active_lineages: usize = 0;

        for y in landscape_extent.y()..(landscape_extent.y() + landscape_extent.height()) {
            for x in landscape_extent.x()..(landscape_extent.x() + landscape_extent.width()) {
                let location = Location::new(x, y);

                let number_active_lineages_at_location = lineage_store
                    .get_active_lineages_at_location(&location)
                    .len();

                if number_active_lineages_at_location > 0 {
                    let event_rate_at_location = event_sampler.get_event_rate_at_location(
                        &location,
                        &partial_simulation,
                        true,
                    );

                    active_locations.push((
                        location,
                        EventTime::from(rng.sample_exponential(event_rate_at_location)),
                    ));

                    number_active_lineages += number_active_lineages_at_location;
                }
            }
        }

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
