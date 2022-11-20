use necsim_core::{impl_finalise, impl_report, reporter::Reporter};

use super::IndividualSpeciesSQLiteReporter;

impl Reporter for IndividualSpeciesSQLiteReporter {
    impl_report!(speciation(&mut self, speciation: Used) {
        if speciation.prior_time == 0.0_f64 {
            self.store_individual_origin(&speciation.global_lineage_reference, &speciation.origin);
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
        if dispersal.prior_time == 0.0_f64 {
            self.store_individual_origin(&dispersal.global_lineage_reference, &dispersal.origin);
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
        let table = self.table.clone();
        let output = self.output.clone();

        if let Err(err) = self.output_to_database() {
            error!("Failed to write the lineage locations to table {table:?} at {output:?}:\n{err}");
        }
    });

    fn initialise(&mut self) -> Result<(), String> {
        self.initialise_sqlite_connection().map_err(|err| {
            format!("Failed to initialise the SQLite species location list:\n{err}",)
        })
    }
}
