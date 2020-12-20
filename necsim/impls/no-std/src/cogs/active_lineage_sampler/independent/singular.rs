use necsim_core::cogs::{
    DispersalSampler, HabitatToU64Injection, IncoherentLineageStore, LineageReference,
    PrimeableRng, SingularActiveLineageSampler,
};

use crate::cogs::{
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    event_sampler::independent::IndependentEventSampler,
};

use super::{EventTimeSampler, IndependentActiveLineageSampler};

impl<
        H: HabitatToU64Injection,
        G: PrimeableRng<H>,
        T: EventTimeSampler<H, G>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
    >
    SingularActiveLineageSampler<
        H,
        G,
        D,
        R,
        S,
        IndependentCoalescenceSampler<H, G, R, S>,
        IndependentEventSampler<H, G, D, R, S>,
    > for IndependentActiveLineageSampler<H, G, T, D, R, S>
{
    #[must_use]
    fn replace_active_lineage(
        &mut self,
        active_lineage_reference: Option<R>,
        lineage_store: &mut S,
    ) -> Option<R> {
        let old_lineage_reference =
            core::mem::replace(&mut self.active_lineage_reference, active_lineage_reference);

        // Save the state of the old lineage reference back to the lineage store
        if let Some(lineage_reference) = &old_lineage_reference {
            lineage_store.update_lineage_time_of_last_event(
                lineage_reference.clone(),
                self.lineage_time_of_last_event,
            );
            self.lineage_time_of_last_event = 0.0_f64;

            if let Some(indexed_location) = self.lineage_indexed_location.take() {
                lineage_store.insert_lineage_to_indexed_location(
                    lineage_reference.clone(),
                    indexed_location,
                );
            }
        }

        // Load the state of the new lineage reference from the lineage store
        if let Some(lineage_reference) = &self.active_lineage_reference {
            if let Some(lineage) = lineage_store.get(lineage_reference.clone()) {
                if lineage.is_active() {
                    self.lineage_time_of_last_event = lineage.time_of_last_event();
                    self.lineage_indexed_location = Some(
                        lineage_store.extract_lineage_from_its_location(lineage_reference.clone()),
                    );
                }
            }
        }

        old_lineage_reference
    }
}
