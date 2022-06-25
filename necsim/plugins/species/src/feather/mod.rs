use std::{
    collections::{hash_map::Entry, HashMap},
    convert::TryFrom,
    fmt,
    fs::File,
    io::BufReader,
    path::PathBuf,
};

use arrow2::{
    array::{FixedSizeBinaryArray, PrimitiveArray},
    datatypes::{DataType, Field},
};
use fnv::FnvBuildHasher;
use partitions::PartitionVec;
use serde::{Deserialize, Deserializer, Serialize};

use necsim_core::{
    event::{DispersalEvent, SpeciationEvent},
    landscape::Location,
    lineage::GlobalLineageReference,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use crate::{LastEventState, SpeciesIdentity};

mod dataframe;
mod reporter;

#[allow(clippy::module_name_repetitions)]
pub struct LocationGroupedSpeciesReporter {
    last_parent_prior_time: Option<(GlobalLineageReference, NonNegativeF64)>,
    last_speciation_event: Option<SpeciationEvent>,
    last_dispersal_event: Option<DispersalEvent>,

    // Representation counts for all resumed lineages
    counts: HashMap<GlobalLineageReference, u64, FnvBuildHasher>,
    // Last event time of all lineages
    activity: HashMap<GlobalLineageReference, PositiveF64, FnvBuildHasher>,
    // Original (present-time) locations of all lineages
    origins: HashMap<GlobalLineageReference, Location, FnvBuildHasher>,

    // Indices into the union-find PartitionVec
    indices: HashMap<GlobalLineageReference, usize, FnvBuildHasher>,
    // Species-unions of all lineages
    unions: PartitionVec<GlobalLineageReference>,

    // Species originator -> Species identity mapping
    species: HashMap<GlobalLineageReference, SpeciesIdentity, FnvBuildHasher>,
    // All speciated location-species records from before a resume
    speciated: Vec<(Location, SpeciesIdentity, u64)>,

    output: PathBuf,
    mode: SpeciesLocationsMode,
    init: bool,
}

impl Drop for LocationGroupedSpeciesReporter {
    fn drop(&mut self) {
        if matches!(self.mode, SpeciesLocationsMode::Create) && !self.init {
            std::mem::drop(std::fs::remove_file(&self.output));
        }
    }
}

impl fmt::Debug for LocationGroupedSpeciesReporter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(IndividualLocationSpeciesReporter))
            .field("output", &self.output)
            .field("mode", &self.mode)
            .finish()
    }
}

impl serde::Serialize for LocationGroupedSpeciesReporter {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        IndividualLocationSpeciesReporterArgs {
            output: self.output.clone(),
            mode: self.mode.clone(),
        }
        .serialize(serializer)
    }
}

#[allow(clippy::too_many_lines)]
impl<'de> Deserialize<'de> for LocationGroupedSpeciesReporter {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let args = IndividualLocationSpeciesReporterArgs::deserialize(deserializer)?;

        let mut self_last_parent_prior_time = None;
        let mut self_last_speciation_event = None;
        let mut self_last_dispersal_event = None;

        let mut self_counts = HashMap::default();
        let mut self_activity = HashMap::default();
        let mut self_origins = HashMap::default();

        let mut self_indices = HashMap::default();
        let mut self_unions = PartitionVec::default();

        let mut self_speciated = Vec::default();

        if matches!(args.mode, SpeciesLocationsMode::Resume) {
            let file = File::options()
                .read(true)
                .open(&args.output)
                .map_err(serde::de::Error::custom)?;
            let mut reader = BufReader::new(file);

            let metadata = arrow2::io::ipc::read::read_file_metadata(&mut reader)
                .map_err(serde::de::Error::custom)?;

            let expected_fields = vec![
                Field::new("x", DataType::UInt32, false),
                Field::new("y", DataType::UInt32, false),
                Field::new("species", DataType::FixedSizeBinary(24), false),
                Field::new("count", DataType::UInt64, false),
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

                let (xs, ys, species, counts) = match chunk.columns() {
                    [xs, ys, species, counts] => (xs, ys, species, counts),
                    _ => {
                        return Err(serde::de::Error::custom(
                            "corrupted species dataframe schema",
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

                let species = match species.as_any().downcast_ref::<FixedSizeBinaryArray>() {
                    Some(species) if species.size() == 24 => species,
                    _ => {
                        return Err(serde::de::Error::custom(
                            "corrupted species dataframe species column",
                        ))
                    },
                };

                let counts = match counts.as_any().downcast_ref::<PrimitiveArray<u64>>() {
                    Some(counts) => counts,
                    None => {
                        return Err(serde::de::Error::custom(
                            "corrupted species dataframe count column",
                        ))
                    },
                };

                for (((x, y), species), count) in xs
                    .values_iter()
                    .zip(ys.values_iter())
                    .zip(species.iter_values())
                    .zip(counts.values_iter())
                {
                    let origin = Location::new(*x, *y);
                    let species = SpeciesIdentity::try_from(species).map_err(|_| {
                        serde::de::Error::custom("corrupted species dataframe species value")
                    })?;
                    let count = *count;

                    match species.try_into_unspeciated() {
                        Ok((lineage, activity, anchor)) => {
                            match self_counts.entry(lineage.clone()) {
                                Entry::Occupied(_) => {
                                    return Err(serde::de::Error::custom(
                                        "resuming duplicate lineage",
                                    ))
                                },
                                Entry::Vacant(vacant) => vacant.insert(count),
                            };

                            self_activity.insert(lineage.clone(), activity);
                            self_origins.insert(lineage.clone(), origin);

                            let lineage_index =
                                *self_indices.entry(lineage.clone()).or_insert_with(|| {
                                    let index = self_unions.len();
                                    self_unions.push(lineage.clone());
                                    index
                                });

                            if anchor != lineage {
                                let anchor_index =
                                    *self_indices.entry(anchor.clone()).or_insert_with(|| {
                                        let index = self_unions.len();
                                        self_unions.push(anchor);
                                        index
                                    });

                                self_unions.union(lineage_index, anchor_index);
                            }
                        },
                        Err(species) => {
                            self_speciated.push((origin, species, count));
                        },
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

            counts: self_counts,
            activity: self_activity,
            origins: self_origins,

            indices: self_indices,
            unions: self_unions,

            species: HashMap::default(),
            speciated: self_speciated,

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
