use necsim_core::{impl_finalise, impl_report, reporter::Reporter};

use super::LocationSpeciesFeatherReporter;

impl Reporter for LocationSpeciesFeatherReporter {
    impl_report!(speciation(&mut self, speciation: Used) {
        self.init = true;

        if speciation.prior_time == 0.0_f64 {
            self.store_individual_origin(&speciation.global_lineage_reference, speciation.origin.location().clone());
        }

        if Some(speciation) == self.last_speciation_event.as_ref() {
            if let Some((parent, prior_time)) = &self.last_parent_prior_time {
                if prior_time != &speciation.prior_time {
                    let parent = parent.clone();
                    self.store_individual_coalescence(&speciation.global_lineage_reference, &parent);
                }
            }
        } else {
            self.store_individual_speciation(&speciation.global_lineage_reference, &speciation.origin, speciation.event_time);
        }

        self.last_speciation_event = Some(speciation.clone());
        self.last_parent_prior_time = Some(
            (speciation.global_lineage_reference.clone(), speciation.prior_time)
        );
    });

    impl_report!(dispersal(&mut self, dispersal: Used) {
        self.init = true;

        if dispersal.prior_time == 0.0_f64 {
            self.store_individual_origin(&dispersal.global_lineage_reference, dispersal.origin.location().clone());
        }

        // Only update the active frontier with `deduplication_probability`
        // All probabilities result in fully correct results,
        //  but higher probabilities reduce the dataframe size between
        //  pause and resume with the Independent algorithm
        if self.deduplication_probability > dispersal.event_time.get().fract() {
            self.activity.insert(dispersal.global_lineage_reference.clone(), dispersal.event_time);
        }

        if Some(dispersal) == self.last_dispersal_event.as_ref() {
            if let Some((parent, prior_time)) = &self.last_parent_prior_time {
                if prior_time != &dispersal.prior_time {
                    let parent = parent.clone();
                    self.store_individual_coalescence(&dispersal.global_lineage_reference, &parent);
                }
            }
        } else if let Some(ref parent) = dispersal.interaction.parent() {
            self.store_individual_coalescence(&dispersal.global_lineage_reference, parent);
        }

        self.last_dispersal_event = Some(dispersal.clone());
        self.last_parent_prior_time = Some(
            (dispersal.global_lineage_reference.clone(), dispersal.prior_time)
        );
    });

    impl_report!(progress(&mut self, _progress: Ignored) {});

    impl_finalise!((mut self) {
        let output = self.output.clone();

        if let Err(err) = self.output_to_dataframe() {
            error!("Failed to write the species dataframe to {output:?}:\n{err}");
        }
    });

    fn initialise(&mut self) -> Result<(), String> {
        self.init = true;

        Ok(())
    }
}
