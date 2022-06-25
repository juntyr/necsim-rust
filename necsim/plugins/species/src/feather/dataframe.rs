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

use crate::{LastEventState, SpeciesIdentity};

use super::LocationGroupedSpeciesReporter;

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
            SpeciesIdentity::from_speciation(origin, time),
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
            species_index.insert((origin.clone(), identity.clone()), counts.len());

            xs.push(origin.x());
            ys.push(origin.y());
            species.extend_from_slice(&*identity);
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
                    species = Some(identity.clone());
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

                        match species_index.entry((origin.clone(), identity.clone())) {
                            // Update the existing per-location-species record
                            Entry::Occupied(occupied) => counts[*occupied.get()] += count,
                            // Create a new per-location-species record
                            Entry::Vacant(vacant) => {
                                vacant.insert(counts.len());

                                xs.push(origin.x());
                                ys.push(origin.y());
                                species.extend_from_slice(&*identity);
                                counts.push(count);
                            },
                        }
                    }
                }
            } else if let Some((anchor_activity, anchor)) = activity {
                // The unspeciated species-union has one fallback anchor
                let anchor_identity = SpeciesIdentity::from_unspeciated(
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
                                species.extend_from_slice(&*SpeciesIdentity::from_unspeciated(
                                    lineage.clone(),
                                    activity,
                                    anchor.clone(),
                                ));
                                counts.push(count);

                                continue;
                            }
                        }

                        // No-longer activate lineages and the anchor may share
                        //  location-species records with each other
                        match species_index.entry((origin.clone(), anchor_identity.clone())) {
                            // Update the existing per-location-species record
                            Entry::Occupied(occupied) => counts[*occupied.get()] += count,
                            // Create a new per-location-species record
                            Entry::Vacant(vacant) => {
                                vacant.insert(counts.len());

                                xs.push(origin.x());
                                ys.push(origin.y());
                                species.extend_from_slice(&*anchor_identity);
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
