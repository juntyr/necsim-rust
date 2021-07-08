use std::{collections::HashMap, convert::TryFrom, fmt, fs::OpenOptions, io};

use serde::{Deserialize, Serialize, Serializer};
use tskit::TableCollection;

use necsim_core::{
    event::{DispersalEvent, SpeciationEvent},
    landscape::IndexedLocation,
    lineage::GlobalLineageReference,
};
use necsim_core_bond::NonNegativeF64;

mod metadata;
mod reporter;
mod table;

// An arbitrary genome sequence interval
const TSK_SEQUENCE_MIN: f64 = 0.0_f64;
const TSK_SEQUENCE_MAX: f64 = 1.0_f64;

#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize)]
#[serde(try_from = "TskitTreeReporterArgs")]
pub struct TskitTreeReporter {
    last_parent_prior_time: Option<(GlobalLineageReference, NonNegativeF64)>,
    last_speciation_event: Option<SpeciationEvent>,
    last_dispersal_event: Option<DispersalEvent>,

    // Original (present-time) locations of all lineages
    origins: HashMap<GlobalLineageReference, IndexedLocation>,
    // Children lineages of all parents, used to create tskit individuals in order
    children: HashMap<GlobalLineageReference, Vec<(GlobalLineageReference, f64)>>,

    table: TableCollection,

    output: String,
}

impl Serialize for TskitTreeReporter {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        TskitTreeReporterArgs {
            output: self.output.clone(),
        }
        .serialize(serializer)
    }
}

impl fmt::Debug for TskitTreeReporter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("TskitTreeReporter")
            .field("output", &self.output)
            .finish()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename = "TskitTreeReporter")]
struct TskitTreeReporterArgs {
    output: String,
}

impl TryFrom<TskitTreeReporterArgs> for TskitTreeReporter {
    type Error = io::Error;

    fn try_from(args: TskitTreeReporterArgs) -> Result<Self, Self::Error> {
        // Preliminary argument parsing check if the output is a writable file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&args.output)?;
        std::mem::drop(file);

        let table = TableCollection::new(TSK_SEQUENCE_MAX)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

        Ok(Self {
            last_parent_prior_time: None,
            last_speciation_event: None,
            last_dispersal_event: None,

            origins: HashMap::new(),
            children: HashMap::new(),

            table,

            output: args.output,
        })
    }
}
