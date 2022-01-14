use std::{collections::HashMap, fmt, path::PathBuf};

use fnv::FnvBuildHasher;
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Deserializer, Serialize};

use necsim_core::{
    event::{DispersalEvent, SpeciationEvent},
    landscape::IndexedLocation,
    lineage::GlobalLineageReference,
};
use necsim_core_bond::NonNegativeF64;

mod database;
mod reporter;

#[derive(Debug)]
struct SpeciesIdentity(u64, u64, u64);

#[allow(clippy::module_name_repetitions)]
pub struct SpeciesLocationsReporter {
    last_parent_prior_time: Option<(GlobalLineageReference, NonNegativeF64)>,
    last_speciation_event: Option<SpeciationEvent>,
    last_dispersal_event: Option<DispersalEvent>,

    // Original (present-time) locations of all lineages
    origins: HashMap<GlobalLineageReference, IndexedLocation, FnvBuildHasher>,
    // Child -> Parent lineage mapping
    parents: HashMap<GlobalLineageReference, GlobalLineageReference, FnvBuildHasher>,
    // Species originator -> Species identities mapping
    species: HashMap<GlobalLineageReference, SpeciesIdentity, FnvBuildHasher>,

    output: PathBuf,
    table: String,
    mode: SpeciesLocationsMode,

    connection: Connection,
}

impl fmt::Debug for SpeciesLocationsReporter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(SpeciesLocationsReporter))
            .field("output", &self.output)
            .field("table", &self.table)
            .finish()
    }
}

impl serde::Serialize for SpeciesLocationsReporter {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        SpeciesLocationsReporterArgs {
            output: self.output.clone(),
            table: self.table.clone(),
            mode: self.mode.clone(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SpeciesLocationsReporter {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let args = SpeciesLocationsReporterArgs::deserialize(deserializer)?;

        let connection = Connection::open_with_flags(
            &args.output,
            match args.mode {
                SpeciesLocationsMode::Resume => OpenFlags::SQLITE_OPEN_READ_WRITE,
                SpeciesLocationsMode::Create => {
                    OpenFlags::SQLITE_OPEN_CREATE | OpenFlags::SQLITE_OPEN_READ_WRITE
                },
            },
        )
        .map_err(serde::de::Error::custom)?;

        Ok(Self {
            last_parent_prior_time: None,
            last_speciation_event: None,
            last_dispersal_event: None,

            origins: HashMap::default(),
            parents: HashMap::default(),
            species: HashMap::default(),

            output: args.output,
            table: args.table,
            mode: args.mode,

            connection,
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SpeciesLocationsReporterArgs {
    output: PathBuf,
    #[serde(default = "default_table_name")]
    table: String,
    #[serde(default)]
    mode: SpeciesLocationsMode,
}

fn default_table_name() -> String {
    String::from("SPECIES_LOCATIONS")
}

#[derive(Clone, Serialize, Deserialize)]
enum SpeciesLocationsMode {
    Create,
    Resume,
}

impl Default for SpeciesLocationsMode {
    fn default() -> Self {
        SpeciesLocationsMode::Create
    }
}
