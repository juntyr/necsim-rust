use necsim_core::cogs::{
    DispersalSampler, EmigrationExit, Habitat, IncoherentLineageStore, LineageReference,
    PrimeableRng, SingularActiveLineageSampler, SpeciationProbability,
};

use crate::cogs::{
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    event_sampler::independent::IndependentEventSampler,
    immigration_entry::never::NeverImmigrationEntry,
};

use super::{EventTimeSampler, IndependentActiveLineageSampler};

impl<
        H: Habitat,
        G: PrimeableRng<H>,
        N: SpeciationProbability<H>,
        T: EventTimeSampler<H, G>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
    >
    SingularActiveLineageSampler<
        H,
        G,
        N,
        D,
        R,
        S,
        X,
        IndependentCoalescenceSampler<H, R, S>,
        IndependentEventSampler<H, G, N, D, R, S, X>,
        NeverImmigrationEntry,
    > for IndependentActiveLineageSampler<H, G, N, T, D, R, S, X>
{
    #[must_use]
    #[inline]
    fn replace_active_lineage(
        &mut self,
        active_lineage_reference: Option<R>,
        habitat: &H,
        lineage_store: &mut S,
    ) -> Option<R> {
        let old_lineage_reference = self.active_lineage_reference.take();

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
                    habitat,
                );
            }
        }

        // Load the state of the new lineage reference from the lineage store
        if let Some(lineage_reference) = active_lineage_reference {
            if let Some(lineage) = lineage_store.get(lineage_reference.clone()) {
                if lineage.is_active() {
                    self.lineage_time_of_last_event = lineage.time_of_last_event();
                    self.lineage_indexed_location = Some(
                        lineage_store
                            .extract_lineage_from_its_location(lineage_reference.clone(), habitat),
                    );
                    self.active_lineage_reference = Some(lineage_reference);
                }
            }
        }

        old_lineage_reference
    }
}
