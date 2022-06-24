use necsim_core::{impl_finalise, impl_report, reporter::Reporter};

use super::LocationGroupedSpeciesReporter;

impl Reporter for LocationGroupedSpeciesReporter {
    impl_report!(speciation(&mut self, speciation: Used) {
        self.init = true;

        if speciation.prior_time == 0.0_f64 {
            self.store_individual_origin(&speciation.global_lineage_reference, speciation.origin.location());
        }

        // No activity is needed for speciated individuals
        self.activity.remove(&speciation.global_lineage_reference);

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
            self.store_individual_origin(&dispersal.global_lineage_reference, dispersal.origin.location());
        }

        if dispersal.interaction.is_coalescence() {
            // Definitely coalesced individuals must NOT have an activity
            self.activity.remove(&dispersal.global_lineage_reference);
        } else {
            // Store the latest event time of the lineage
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
            error!("Failed to write the species dataframe to {:?}:\n{}", output, err);
        }
    });

    fn initialise(&mut self) -> Result<(), String> {
        self.init = true;

        Ok(())
    }
}
