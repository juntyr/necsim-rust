use std::{collections::HashMap, convert::TryFrom, fmt, path::PathBuf};

use rusqlite::Connection;
use serde::Deserialize;

use necsim_core::{
    event::{DispersalEvent, SpeciationEvent},
    landscape::IndexedLocation,
    lineage::GlobalLineageReference,
};
use necsim_core_bond::NonNegativeF64;

mod database;
mod reporter;

struct SpeciesIdentity(u64, u64, u64);

#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize)]
#[serde(try_from = "SpeciesLocationsReporterArgs")]
pub struct SpeciesLocationsReporter {
    last_parent_prior_time: Option<(GlobalLineageReference, NonNegativeF64)>,
    last_speciation_event: Option<SpeciationEvent>,
    last_dispersal_event: Option<DispersalEvent>,

    // Original (present-time) locations of all lineages
    origins: Vec<(GlobalLineageReference, IndexedLocation)>,
    // Child -> Parent lineage mapping
    parents: HashMap<GlobalLineageReference, GlobalLineageReference>,
    // Species originator -> Species identities mapping
    species: HashMap<GlobalLineageReference, SpeciesIdentity>,

    output: PathBuf,
    table: String,

    connection: Connection,
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

            origins: Vec::new(),
            parents: HashMap::new(),
            species: HashMap::new(),

            output: args.output,
            table: args.table,

            connection,
        })
    }
}
