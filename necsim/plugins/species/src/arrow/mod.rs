use std::{collections::HashMap, convert::TryInto, fmt, fs::File, io::BufReader, path::PathBuf};

use arrow2::{
    array::{FixedSizeBinaryArray, PrimitiveArray},
    datatypes::{DataType, Field},
};
use bincode::Options;
use fnv::FnvBuildHasher;
use serde::{Deserialize, Deserializer, Serialize};

use necsim_core::{
    event::{DispersalEvent, SpeciationEvent},
    landscape::{IndexedLocation, Location},
    lineage::GlobalLineageReference,
};
use necsim_core_bond::{NonNegativeF64, NonZeroOneU64};

mod dataframe;
mod reporter;

#[derive(Debug)]
struct SpeciesIdentity([u8; 24]);

#[allow(clippy::module_name_repetitions)]
pub struct IndividualLocationSpeciesReporter {
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
    mode: SpeciesLocationsMode,
    init: bool,
}

impl Drop for IndividualLocationSpeciesReporter {
    fn drop(&mut self) {
        if matches!(self.mode, SpeciesLocationsMode::Create) && !self.init {
            std::mem::drop(std::fs::remove_file(&self.output));
        }
    }
}

impl fmt::Debug for IndividualLocationSpeciesReporter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(IndividualLocationSpeciesReporter))
            .field("output", &self.output)
            .field("mode", &self.mode)
            .finish()
    }
}

impl serde::Serialize for IndividualLocationSpeciesReporter {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        IndividualLocationSpeciesReporterArgs {
            output: self.output.clone(),
            mode: self.mode.clone(),
        }
        .serialize(serializer)
    }
}

#[allow(clippy::too_many_lines)]
impl<'de> Deserialize<'de> for IndividualLocationSpeciesReporter {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let args = IndividualLocationSpeciesReporterArgs::deserialize(deserializer)?;

        let mut self_last_parent_prior_time = None;
        let mut self_last_speciation_event = None;
        let mut self_last_dispersal_event = None;

        let mut self_origins = HashMap::default();
        let mut self_parents = HashMap::default();
        let mut self_species = HashMap::default();

        if matches!(args.mode, SpeciesLocationsMode::Resume) {
            let file = File::options()
                .read(true)
                .open(&args.output)
                .map_err(serde::de::Error::custom)?;
            let mut reader = BufReader::new(file);

            let metadata = arrow2::io::ipc::read::read_file_metadata(&mut reader)
                .map_err(serde::de::Error::custom)?;

            let expected_fields = vec![
                Field::new("id", DataType::UInt64, false),
                Field::new("x", DataType::UInt32, false),
                Field::new("y", DataType::UInt32, false),
                Field::new("i", DataType::UInt32, false),
                Field::new("parent", DataType::UInt64, false),
                Field::new("species", DataType::FixedSizeBinary(24), true),
            ];

            if metadata.schema.fields != expected_fields {
                return Err(serde::de::Error::custom(
                    "species dataframe schema mismatch",
                ));
            }

            let last_event = match metadata.schema.metadata.get("last-event") {
                Some(last_event) => LastEventState::from_string(last_event).map_err(|_| {
                    serde::de::Error::custom("invalid resume metadata in species dataframe")
                })?,
                None => {
                    return Err(serde::de::Error::custom(
                        "resume metadata missing from species dataframe",
                    ))
                },
            };

            self_last_parent_prior_time = last_event.last_parent_prior_time;
            self_last_speciation_event = last_event.last_speciation_event;
            self_last_dispersal_event = last_event.last_dispersal_event;

            for chunk in arrow2::io::ipc::read::FileReader::new(reader, metadata, None) {
                let chunk = chunk.map_err(serde::de::Error::custom)?;

                let (ids, xs, ys, is, parents, species) = match chunk.columns() {
                    [ids, xs, ys, is, parents, species] => (ids, xs, ys, is, parents, species),
                    _ => {
                        return Err(serde::de::Error::custom(
                            "corrupted species dataframe schema",
                        ))
                    },
                };

                let ids = match ids.as_any().downcast_ref::<PrimitiveArray<u64>>() {
                    Some(ids) => ids,
                    None => {
                        return Err(serde::de::Error::custom(
                            "corrupted species dataframe id column",
                        ))
                    },
                };

                let xs = match xs.as_any().downcast_ref::<PrimitiveArray<u32>>() {
                    Some(xs) => xs,
                    None => {
                        return Err(serde::de::Error::custom(
                            "corrupted species dataframe x column",
                        ))
                    },
                };

                let ys = match ys.as_any().downcast_ref::<PrimitiveArray<u32>>() {
                    Some(ys) => ys,
                    None => {
                        return Err(serde::de::Error::custom(
                            "corrupted species dataframe y column",
                        ))
                    },
                };

                let is = match is.as_any().downcast_ref::<PrimitiveArray<u32>>() {
                    Some(is) => is,
                    None => {
                        return Err(serde::de::Error::custom(
                            "corrupted species dataframe i column",
                        ))
                    },
                };

                let parents = match parents.as_any().downcast_ref::<PrimitiveArray<u64>>() {
                    Some(parents) => parents,
                    None => {
                        return Err(serde::de::Error::custom(
                            "corrupted species dataframe parent column",
                        ))
                    },
                };

                let species = match species.as_any().downcast_ref::<FixedSizeBinaryArray>() {
                    Some(species) if species.size() == 24 => species,
                    _ => {
                        return Err(serde::de::Error::custom(
                            "corrupted species dataframe species column",
                        ))
                    },
                };

                for (((((id, x), y), i), parent), species) in ids
                    .values_iter()
                    .zip(xs.values_iter())
                    .zip(ys.values_iter())
                    .zip(is.values_iter())
                    .zip(parents.values_iter())
                    .zip(species.iter())
                {
                    let id = unsafe {
                        GlobalLineageReference::from_inner(NonZeroOneU64::new_unchecked(*id + 2))
                    };

                    // Populate the individual `origins` lookup
                    self_origins
                        .insert(id.clone(), IndexedLocation::new(Location::new(*x, *y), *i));

                    let parent = unsafe {
                        GlobalLineageReference::from_inner(NonZeroOneU64::new_unchecked(
                            *parent + 2,
                        ))
                    };

                    // Populate the individual `parents` lookup
                    // parent == id -> individual does NOT have a parent
                    if parent != id {
                        self_parents.insert(id.clone(), parent);
                    }

                    if let Some(species) = species {
                        let species: [u8; 24] = species.try_into().map_err(|_| {
                            serde::de::Error::custom("corrupted species dataframe species value")
                        })?;

                        // Populate the individual `species` lookup
                        self_species.insert(id, SpeciesIdentity(species));
                    }
                }
            }
        } else {
            File::options()
                .create_new(true)
                .write(true)
                .open(&args.output)
                .map_err(serde::de::Error::custom)?;
        }

        Ok(Self {
            last_parent_prior_time: self_last_parent_prior_time,
            last_speciation_event: self_last_speciation_event,
            last_dispersal_event: self_last_dispersal_event,

            origins: self_origins,
            parents: self_parents,
            species: self_species,

            output: args.output,
            mode: args.mode,
            init: false,
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename = "IndividualLocationSpeciesReporter")]
struct IndividualLocationSpeciesReporterArgs {
    output: PathBuf,
    #[serde(default)]
    mode: SpeciesLocationsMode,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum SpeciesLocationsMode {
    Create,
    Resume,
}

impl Default for SpeciesLocationsMode {
    fn default() -> Self {
        SpeciesLocationsMode::Create
    }
}

#[derive(Serialize, Deserialize)]
struct LastEventState {
    last_parent_prior_time: Option<(GlobalLineageReference, NonNegativeF64)>,
    last_speciation_event: Option<SpeciationEvent>,
    last_dispersal_event: Option<DispersalEvent>,
}

impl LastEventState {
    fn into_string(self) -> Result<String, ()> {
        let bytes = bincode::options().serialize(&self).map_err(|_| ())?;

        Ok(base32::encode(base32::Alphabet::Crockford, &bytes))
    }

    fn from_string(string: &str) -> Result<LastEventState, ()> {
        let bytes = base32::decode(base32::Alphabet::Crockford, string).ok_or(())?;

        bincode::options().deserialize(&bytes).map_err(|_| ())
    }
}
