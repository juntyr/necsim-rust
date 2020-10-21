use std::marker::PhantomData;

use necsim_core::landscape::Landscape;
use necsim_core::lineage::LineageReference;
use necsim_core::rng::Rng;
use necsim_core::simulation::SimulationSettings;

use crate::event_generator::coalescence_sampler::ConditionalCoalescenceSampler;
use crate::event_generator::event_type_sampler::unconditional::UnconditionalEventTypeSampler;
use crate::event_generator::lineage_sampler::LineageSampler;

mod event_generator;

#[allow(clippy::module_name_repetitions)]
pub struct UnconditionalGlobalEventGenerator<
    L: LineageReference,
    S: LineageSampler<L> + ConditionalCoalescenceSampler<L>,
> {
    event_type_sampler: UnconditionalEventTypeSampler,
    lineage_coalescence_sampler: S,
    lineage_reference: PhantomData<L>,
}

impl<L: LineageReference, S: LineageSampler<L> + ConditionalCoalescenceSampler<L>>
    UnconditionalGlobalEventGenerator<L, S>
{
    pub fn new(lineage_coalescence_sampler: S) -> Self {
        Self {
            event_type_sampler: UnconditionalEventTypeSampler,
            lineage_coalescence_sampler,
            lineage_reference: PhantomData,
        }
    }

    #[debug_ensures(ret >= 0.0_f64, "delta_time sample is non-negative")]
    fn sample_final_speciation_delta_time(
        settings: &SimulationSettings<impl Landscape>,
        rng: &mut impl Rng,
    ) -> f64 {
        rng.sample_exponential(0.5_f64 * settings.speciation_probability_per_generation())
    }
}
