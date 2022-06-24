use std::{
    collections::{hash_map::Entry, BTreeMap, HashMap},
    fs::File,
    io::BufWriter,
};

use arrow2::{
    array::{FixedSizeBinaryArray, PrimitiveArray},
    buffer::Buffer,
    chunk::Chunk,
    datatypes::{DataType, Field, Schema},
    io::ipc::write::{FileWriter, WriteOptions},
};
use fnv::FnvBuildHasher;
use necsim_core::{
    landscape::{IndexedLocation, Location},
    lineage::GlobalLineageReference,
};
use necsim_core_bond::PositiveF64;

use super::{LastEventState, LocationGroupedSpeciesReporter, SpeciesIdentity};

impl LocationGroupedSpeciesReporter {
    pub(super) fn store_individual_origin(
        &mut self,
        lineage: &GlobalLineageReference,
        origin: &Location,
    ) {
        self.origins.insert(lineage.clone(), origin.clone());
    }

    pub(super) fn store_individual_speciation(
        &mut self,
        lineage: &GlobalLineageReference,
        origin: &IndexedLocation,
        time: PositiveF64,
    ) {
        if let Entry::Vacant(vacant) = self.indices.entry(lineage.clone()) {
            vacant.insert(self.unions.len());
            self.unions.push(lineage.clone());
        }

        self.species.insert(
            lineage.clone(),
            create_species_identity_from_speciation(origin, time),
        );
    }

    pub(super) fn store_individual_coalescence(
        &mut self,
        child: &GlobalLineageReference,
        parent: &GlobalLineageReference,
    ) {
        let child_index = match self.indices.entry(child.clone()) {
            Entry::Occupied(occupied) => *occupied.get(),
            Entry::Vacant(vacant) => {
                let index = *vacant.insert(self.unions.len());
                self.unions.push(child.clone());
                index
            },
        };

        let parent_index = match self.indices.entry(parent.clone()) {
            Entry::Occupied(occupied) => *occupied.get(),
            Entry::Vacant(vacant) => {
                let index = *vacant.insert(self.unions.len());
                self.unions.push(parent.clone());
                index
            },
        };

        self.unions.union(child_index, parent_index);
    }

    #[allow(clippy::too_many_lines)]
    pub(super) fn output_to_dataframe(mut self) -> arrow2::error::Result<()> {
        let file = File::options()
            .write(true)
            .truncate(true)
            .open(&self.output)?;
        let writer = BufWriter::new(file);

        let expected_fields = vec![
            Field::new("x", DataType::UInt32, false),
            Field::new("y", DataType::UInt32, false),
            Field::new("species", DataType::FixedSizeBinary(24), false),
            Field::new("count", DataType::UInt64, false),
        ];

        let mut metadata = BTreeMap::new();
        metadata.insert(
            String::from("last-event"),
            LastEventState {
                last_parent_prior_time: self.last_parent_prior_time.clone(),
                last_speciation_event: self.last_speciation_event.clone(),
                last_dispersal_event: self.last_dispersal_event.clone(),
            }
            .into_string()
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "failed to write metadata to species dataframe",
                )
            })?,
        );

        let mut writer = FileWriter::new(
            writer,
            Schema {
                fields: expected_fields,
                metadata,
            },
            None,
            WriteOptions { compression: None },
        );
        writer.start()?;

        let mut xs = Vec::new();
        let mut ys = Vec::new();
        let mut species = Vec::new();
        let mut counts = Vec::new();

        let mut species_index: HashMap<(Location, SpeciesIdentity), usize, FnvBuildHasher> =
            HashMap::default();

        for (origin, identity, count) in std::mem::take(&mut self.speciated) {
            species_index.insert((origin.clone(), SpeciesIdentity(identity.0)), counts.len());

            xs.push(origin.x());
            ys.push(origin.y());
            species.extend_from_slice(&identity.0);
            counts.push(count);
        }

        let mut union_species = Vec::new();
        let mut union_activity = Vec::new();

        for union in self.unions.all_sets() {
            let mut species = None;
            let mut activity = None;

            for (_, lineage) in union {
                // Find if the union has speciated
                if let Some(identity) = self.species.get(lineage) {
                    species = Some(SpeciesIdentity(identity.0));
                }

                // Find possible union-anchor lineages in the active frontier
                if let Some(a) = self.activity.get(lineage) {
                    if let Some((b, _)) = &activity {
                        if *a > *b {
                            activity = Some((*a, lineage.clone()));
                        }
                    }
                }
            }

            union_species.push(species);
            union_activity.push(activity);
        }

        for ((union, identity), activity) in self
            .unions
            .all_sets()
            .zip(union_species.into_iter())
            .zip(union_activity.into_iter())
        {
            if let Some(identity) = identity {
                // The species-union has already speciated
                for (_, lineage) in union {
                    if let Some(origin) = self.origins.get(lineage) {
                        let count = self.counts.get(lineage).copied().unwrap_or(1_u64);

                        match species_index.entry((origin.clone(), SpeciesIdentity(identity.0))) {
                            // Update the existing per-location-species record
                            Entry::Occupied(occupied) => counts[*occupied.get()] += count,
                            // Create a new per-location-species record
                            Entry::Vacant(vacant) => {
                                vacant.insert(counts.len());

                                xs.push(origin.x());
                                ys.push(origin.y());
                                species.extend_from_slice(&identity.0);
                                counts.push(count);
                            },
                        }
                    }
                }
            } else if let Some((anchor_activity, anchor)) = activity {
                // The unspeciated species-union has one fallback anchor
                let anchor_identity = create_species_identity_from_unspeciated(
                    anchor.clone(),
                    anchor_activity,
                    anchor.clone(),
                );

                for (_, lineage) in union {
                    if let Some(origin) = self.origins.get(lineage) {
                        let count = self.counts.get(lineage).copied().unwrap_or(1_u64);

                        if let Some(activity) = self.activity.get(lineage).copied() {
                            // Lineages in the active frontier of the species-union,
                            //  excluding the anchor, have unique location-species records
                            if activity == anchor_activity && lineage != &anchor {
                                xs.push(origin.x());
                                ys.push(origin.y());
                                species.extend_from_slice(
                                    &create_species_identity_from_unspeciated(
                                        lineage.clone(),
                                        activity,
                                        anchor.clone(),
                                    )
                                    .0,
                                );
                                counts.push(count);

                                continue;
                            }
                        }

                        // No-longer activate lineages and the anchor may share
                        //  location-species records with each other
                        match species_index
                            .entry((origin.clone(), SpeciesIdentity(anchor_identity.0)))
                        {
                            // Update the existing per-location-species record
                            Entry::Occupied(occupied) => counts[*occupied.get()] += count,
                            // Create a new per-location-species record
                            Entry::Vacant(vacant) => {
                                vacant.insert(counts.len());

                                xs.push(origin.x());
                                ys.push(origin.y());
                                species.extend_from_slice(&anchor_identity.0);
                                counts.push(count);
                            },
                        }
                    }
                }
            } else {
                unreachable!("all lineage unions must either speciate or have an active frontier")
            }
        }

        let xs = PrimitiveArray::from_vec(xs);
        let ys = PrimitiveArray::from_vec(ys);
        let species = FixedSizeBinaryArray::try_new(
            DataType::FixedSizeBinary(24),
            Buffer::from(species),
            None,
        )?;
        let counts = PrimitiveArray::from_vec(counts);

        let chunk = Chunk::try_new(vec![
            xs.arced(),
            ys.arced(),
            species.arced(),
            counts.arced(),
        ])?;
        writer.write(&chunk, None)?;

        writer.finish()
    }
}

fn create_species_identity_from_speciation(
    origin: &IndexedLocation,
    time: PositiveF64,
) -> SpeciesIdentity {
    let location = (u64::from(origin.location().y()) << 32) | u64::from(origin.location().x());
    let index = u64::from(origin.index()) << 16;
    let time = time.get().to_bits();

    create_species_identity_from_raw(location, index, time)
}

fn create_species_identity_from_unspeciated(
    lineage: GlobalLineageReference,
    activity: PositiveF64,
    anchor: GlobalLineageReference,
) -> SpeciesIdentity {
    let lineage = unsafe { lineage.into_inner().get() - 2 };

    let anchor = unsafe { anchor.into_inner().get() - 2 };
    assert!(anchor <= (u64::MAX >> 1), "excessive number of species");
    let anchor = (anchor << 1) | 0x1;

    let activity = activity.get().to_bits();

    create_species_identity_from_raw(lineage, anchor, activity)
}

const fn create_species_identity_from_raw(a: u64, b: u64, c: u64) -> SpeciesIdentity {
    let a_bytes = seahash_diffuse(a).to_le_bytes();
    let b_bytes = seahash_diffuse(b).to_le_bytes();
    let c_bytes = seahash_diffuse(c).to_le_bytes();

    // Shuffle and mix all 24 bytes of the species identity
    let lower = seahash_diffuse(u64::from_le_bytes([
        a_bytes[3], c_bytes[0], b_bytes[5], a_bytes[1], c_bytes[4], c_bytes[7], c_bytes[5],
        a_bytes[5],
    ]))
    .to_le_bytes();
    let middle = seahash_diffuse(u64::from_le_bytes([
        c_bytes[6], b_bytes[4], a_bytes[0], a_bytes[6], b_bytes[2], b_bytes[1], a_bytes[7],
        b_bytes[3],
    ]))
    .to_le_bytes();
    let upper = seahash_diffuse(u64::from_le_bytes([
        a_bytes[4], a_bytes[2], c_bytes[2], b_bytes[0], c_bytes[3], c_bytes[1], b_bytes[7],
        b_bytes[6],
    ]))
    .to_le_bytes();

    SpeciesIdentity([
        lower[0], lower[1], lower[2], lower[3], lower[4], lower[5], lower[6], lower[7], middle[0],
        middle[1], middle[2], middle[3], middle[4], middle[5], middle[6], middle[7], upper[0],
        upper[1], upper[2], upper[3], upper[4], upper[5], upper[6], upper[7],
    ])
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
