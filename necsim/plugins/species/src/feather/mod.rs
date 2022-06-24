use std::{
    collections::{hash_map::Entry, HashMap},
    convert::TryInto,
    fmt,
    fs::File,
    io::BufReader,
    path::PathBuf,
};

use arrow2::{
    array::{FixedSizeBinaryArray, PrimitiveArray},
    datatypes::{DataType, Field},
};
use bincode::Options;
use fnv::FnvBuildHasher;
use partitions::PartitionVec;
use serde::{Deserialize, Deserializer, Serialize};

use necsim_core::{
    event::{DispersalEvent, SpeciationEvent},
    landscape::Location,
    lineage::GlobalLineageReference,
};
use necsim_core_bond::{NonNegativeF64, NonZeroOneU64, PositiveF64};

mod dataframe;
mod reporter;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
struct SpeciesIdentity([u8; 24]);

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
                    let species = SpeciesIdentity(species.try_into().map_err(|_| {
                        serde::de::Error::custom("corrupted species dataframe species value")
                    })?);
                    let count = *count;

                    match try_species_identity_into_unspeciated(species) {
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

fn try_species_identity_into_unspeciated(
    identity: SpeciesIdentity,
) -> Result<(GlobalLineageReference, PositiveF64, GlobalLineageReference), SpeciesIdentity> {
    let (lineage, anchor, activity) = species_identity_into_raw(&identity);

    if anchor & 0x1 == 0x0 {
        return Err(identity);
    }

    let lineage =
        unsafe { GlobalLineageReference::from_inner(NonZeroOneU64::new_unchecked(lineage + 2)) };
    let activity = unsafe { PositiveF64::new_unchecked(f64::from_bits(activity)) };
    let anchor =
        unsafe { GlobalLineageReference::from_inner(NonZeroOneU64::new_unchecked(anchor + 2)) };

    Ok((lineage, activity, anchor))
}

const fn species_identity_into_raw(identity: &SpeciesIdentity) -> (u64, u64, u64) {
    let lower_bytes = seahash_undiffuse(u64::from_le_bytes([
        identity.0[0],
        identity.0[1],
        identity.0[2],
        identity.0[3],
        identity.0[4],
        identity.0[5],
        identity.0[6],
        identity.0[7],
    ]))
    .to_le_bytes();
    let middle_bytes = seahash_undiffuse(u64::from_le_bytes([
        identity.0[8],
        identity.0[9],
        identity.0[10],
        identity.0[11],
        identity.0[12],
        identity.0[13],
        identity.0[14],
        identity.0[15],
    ]))
    .to_le_bytes();
    let upper_bytes = seahash_undiffuse(u64::from_le_bytes([
        identity.0[16],
        identity.0[17],
        identity.0[18],
        identity.0[19],
        identity.0[20],
        identity.0[21],
        identity.0[22],
        identity.0[23],
    ]))
    .to_le_bytes();

    let a = seahash_undiffuse(u64::from_le_bytes([
        middle_bytes[2],
        lower_bytes[3],
        upper_bytes[1],
        lower_bytes[0],
        upper_bytes[0],
        lower_bytes[7],
        middle_bytes[3],
        middle_bytes[6],
    ]));
    let b = seahash_undiffuse(u64::from_le_bytes([
        upper_bytes[3],
        middle_bytes[5],
        middle_bytes[4],
        middle_bytes[7],
        middle_bytes[1],
        lower_bytes[2],
        upper_bytes[7],
        upper_bytes[6],
    ]));
    let c = seahash_undiffuse(u64::from_le_bytes([
        lower_bytes[1],
        upper_bytes[5],
        upper_bytes[2],
        upper_bytes[4],
        lower_bytes[4],
        lower_bytes[6],
        middle_bytes[0],
        lower_bytes[5],
    ]));

    (a, b, c)
}

pub const fn seahash_undiffuse(mut x: u64) -> u64 {
    // SeaHash undiffusion function
    // https://docs.rs/seahash/4.1.0/src/seahash/helper.rs.html#94-105

    // 0x2f72b4215a3d8caf is the modular multiplicative inverse of the constant used
    // in `diffuse`.

    x = x.wrapping_mul(0x2f72_b421_5a3d_8caf);

    let a = x >> 32;
    let b = x >> 60;

    x ^= a >> b;

    x = x.wrapping_mul(0x2f72_b421_5a3d_8caf);

    x = x.wrapping_sub(0x9e37_79b9_7f4a_7c15);

    x
}
