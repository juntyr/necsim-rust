use necsim_core::{impl_finalise, impl_report, reporter::Reporter};

use super::TskitTreeReporter;

impl Reporter for TskitTreeReporter {
    impl_report!(speciation(&mut self, speciation: Used) {
        if speciation.prior_time == 0.0_f64 {
            self.store_individual_origin(&speciation.global_lineage_reference, &speciation.origin);
        }

        if Some(speciation) == self.last_speciation_event.as_ref() {
            if let Some((parent, prior_time)) = &self.last_parent_prior_time {
                if prior_time != &speciation.prior_time {
                    let parent = parent.clone();
                    self.store_individual_coalescence(&speciation.global_lineage_reference, &parent, speciation.prior_time);
                }
            }
        } else {
            self.store_individual_speciation(&speciation.global_lineage_reference, speciation.event_time.into());
        }

        self.last_speciation_event = Some(speciation.clone());
        self.last_parent_prior_time = Some(
            (speciation.global_lineage_reference.clone(), speciation.prior_time)
        );
    });

    impl_report!(dispersal(&mut self, dispersal: Used) {
        if dispersal.prior_time == 0.0_f64 {
            self.store_individual_origin(&dispersal.global_lineage_reference, &dispersal.origin);
        }

        if Some(dispersal) == self.last_dispersal_event.as_ref() {
            if let Some((parent, prior_time)) = &self.last_parent_prior_time {
                if prior_time != &dispersal.prior_time {
                    let parent = parent.clone();
                    self.store_individual_coalescence(&dispersal.global_lineage_reference, &parent, dispersal.prior_time);
                }
            }
        } else if let Some(parent) = &dispersal.interaction.parent() {
            self.store_individual_coalescence(&dispersal.global_lineage_reference, parent, dispersal.event_time.into());
        }

        self.last_dispersal_event = Some(dispersal.clone());
        self.last_parent_prior_time = Some(
            (dispersal.global_lineage_reference.clone(), dispersal.prior_time)
        );
    });

    impl_report!(progress(&mut self, _progress: Ignored) {});

    impl_finalise!((mut self) {
        self.output_tree_sequence();
    });

    fn initialise(&mut self) -> Result<(), String> {
        self.store_provenance()
    }
}
