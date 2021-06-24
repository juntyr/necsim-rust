#![deny(clippy::pedantic)]

#[macro_use]
extern crate log;

use std::{convert::TryFrom, fmt, path::PathBuf};

use rusqlite::{params, Connection, Statement};
use serde::Deserialize;

use necsim_core::{
    event::{DispersalEvent, LineageInteraction, SpeciationEvent},
    impl_finalise, impl_report,
    landscape::IndexedLocation,
    lineage::GlobalLineageReference,
    reporter::Reporter,
};
use necsim_core_bond::NonNegativeF64;

necsim_plugins_core::export_plugin!(Species => SpeciesLocationsReporter);

#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize)]
#[serde(try_from = "SpeciesLocationsReporterArgs")]
pub struct SpeciesLocationsReporter {
    last_parent_prior_time: Option<(GlobalLineageReference, NonNegativeF64)>,
    last_speciation_event: Option<SpeciationEvent>,
    last_dispersal_event: Option<DispersalEvent>,

    output: PathBuf,
    table: String,

    connection: Connection,

    insertion: Option<Statement<'static>>,
    speciation: Option<Statement<'static>>,
    coalescence: Option<Statement<'static>>,
}

impl fmt::Debug for SpeciesLocationsReporter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SpeciesLocationsReporter")
            .field("output", &self.output)
            .field("table", &self.table)
            .finish()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpeciesLocationsReporterArgs {
    output: PathBuf,
    #[serde(default = "default_table_name")]
    table: String,
}

fn default_table_name() -> String {
    String::from("SPECIES_LOCATIONS")
}

impl TryFrom<SpeciesLocationsReporterArgs> for SpeciesLocationsReporter {
    type Error = rusqlite::Error;

    fn try_from(args: SpeciesLocationsReporterArgs) -> Result<Self, Self::Error> {
        let connection = Connection::open(&args.output)?;

        Ok(Self {
            last_parent_prior_time: None,
            last_speciation_event: None,
            last_dispersal_event: None,

            output: args.output,
            table: args.table,

            connection,

            insertion: None,
            speciation: None,
            coalescence: None,
        })
    }
}

impl Reporter for SpeciesLocationsReporter {
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
            self.store_individual_speciation(speciation);
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
        } else if let LineageInteraction::Coalescence(parent) = &dispersal.interaction {
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
        let table = self.table.clone();

        if let Err(err) = (|| -> rusqlite::Result<()> {
            if let Some(insertion) = self.insertion.take() {
                insertion.finalize()?;
            }

            if let Some(speciation) = self.speciation.take() {
                speciation.finalize()?;
            }

            if let Some(coalescence) = self.coalescence.take() {
                coalescence.finalize()?;
            }

            self.connection.close().map_err(|(_, err)| err)
        })() {
            error!("Failed to write the lineage locations to table {:?} at {:?}: {:?}", table, output, err);
        }
    });

    fn initialise(&mut self) -> Result<(), String> {
        (|| -> rusqlite::Result<()> {
            self.connection.execute(
                &format!(
                    "CREATE TABLE IF NOT EXISTS {} (
                        id      INTEGER PRIMARY KEY NOT NULL,
                        x       INTEGER NOT NULL,
                        y       INTEGER NOT NULL,
                        i       INTEGER NOT NULL,
                        parent  INTEGER NOT NULL,
                        species TEXT
                    )",
                    self.table
                ),
                [],
            )?;

            self.insertion = Some(extend_statement(self.connection.prepare(&format!(
                "INSERT OR REPLACE INTO {} VALUES (?1, ?2, ?3, ?4, ?1, NULL)",
                self.table,
            ))?));
            self.speciation = Some(extend_statement(self.connection.prepare(&format!(
                "UPDATE {} SET species = ?2 WHERE id = ?1",
                self.table
            ))?));
            self.coalescence = Some(extend_statement(self.connection.prepare(&format!(
                "UPDATE {} SET parent = ?2 WHERE id = ?1",
                self.table
            ))?));

            Ok(())
        })()
        .map_err(|err| {
            format!(
                "Failed to initialise the SQLite species location list: {:?}",
                err
            )
        })
    }
}

impl SpeciesLocationsReporter {
    fn store_individual_origin(
        &mut self,
        individual: &GlobalLineageReference,
        origin: &IndexedLocation,
    ) {
        if let Some(insertion) = &mut self.insertion {
            std::mem::drop(insertion.execute(params![
                individual.to_string(),
                origin.location().x(),
                origin.location().y(),
                origin.index(),
            ]));
        }
    }

    fn store_individual_speciation(&mut self, event: &SpeciationEvent) {
        if let Some(speciation) = &mut self.speciation {
            let location = (u64::from(event.origin.location().y()) << 32)
                | u64::from(event.origin.location().x());
            let index = u64::from(event.origin.index()) << 16;
            let time = event.event_time.get().to_bits();

            let species_id = format!(
                "{:016x}{:016x}{:016x}",
                seahash_diffuse(location),
                seahash_diffuse(index),
                seahash_diffuse(time)
            );

            std::mem::drop(speciation.execute(params![
                event.global_lineage_reference.to_string(),
                species_id,
            ]));
        }
    }

    fn store_individual_coalescence(
        &mut self,
        individual: &GlobalLineageReference,
        parent: &GlobalLineageReference,
    ) {
        if let Some(coalescence) = &mut self.coalescence {
            std::mem::drop(
                coalescence.execute(params![individual.to_string(), parent.to_string(),]),
            );
        }
    }
}

const fn seahash_diffuse(mut x: u64) -> u64 {
    // SeaHash diffusion function
    // https://docs.rs/seahash/4.1.0/src/seahash/helper.rs.html#75-92

    // These are derived from the PCG RNG's round. Thanks to @Veedrac for proposing
    // this. The basic idea is that we use dynamic shifts, which are determined
    // by the input itself. The shift is chosen by the higher bits, which means
    // that changing those flips the lower bits, which scatters upwards because
    // of the multiplication.

    x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);

    x = x.wrapping_mul(0x6eed_0e9d_a4d9_4a4f);

    let a = x >> 32;
    let b = x >> 60;

    x ^= a >> b;

    x = x.wrapping_mul(0x6eed_0e9d_a4d9_4a4f);

    x
}

fn extend_statement<'conn>(statement: Statement<'conn>) -> Statement<'static> {
    unsafe { std::mem::transmute::<Statement<'conn>, Statement<'static>>(statement) }
}
