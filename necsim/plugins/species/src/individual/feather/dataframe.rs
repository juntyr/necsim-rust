use std::{collections::BTreeMap, fs::File, io::BufWriter};

use arrow2::{
    array::{FixedSizeBinaryArray, PrimitiveArray},
    bitmap::MutableBitmap,
    buffer::Buffer,
    chunk::Chunk,
    datatypes::{DataType, Field, Schema},
    io::ipc::write::{FileWriter, WriteOptions},
};
use necsim_core::{landscape::IndexedLocation, lineage::GlobalLineageReference};
use necsim_core_bond::PositiveF64;

use crate::{LastEventState, SpeciesIdentity};

use super::IndividualSpeciesFeatherReporter;

impl IndividualSpeciesFeatherReporter {
    pub(super) fn store_individual_origin(
        &mut self,
        lineage: &GlobalLineageReference,
        origin: &IndexedLocation,
    ) {
        self.origins.insert(lineage.clone(), origin.clone());
    }

    pub(super) fn store_individual_speciation(
        &mut self,
        lineage: &GlobalLineageReference,
        origin: &IndexedLocation,
        time: PositiveF64,
    ) {
        // Resolve the actual parent, irrespective of duplicate individuals
        let mut parent = lineage;
        while let Some(parent_parent) = self.parents.get(parent) {
            parent = parent_parent;
        }

        self.species.insert(
            parent.clone(),
            SpeciesIdentity::from_speciation(origin, time),
        );
    }

    pub(super) fn store_individual_coalescence(
        &mut self,
        child: &GlobalLineageReference,
        parent: &GlobalLineageReference,
    ) {
        // Resolve the actual child, irrespective of duplicate individuals
        let mut child = child;
        while let Some(child_parent) = self.parents.get(child) {
            child = child_parent;
        }
        let child = child.clone();

        // Resolve the actual parent, irrespective of duplicate individuals
        let mut parent = parent;
        while let Some(parent_parent) = self.parents.get(parent) {
            parent = parent_parent;
        }
        let parent = parent.clone();

        // Prevent a lookup-loop, can occur after `Resume`
        if child != parent {
            self.parents.insert(child, parent);
        }
    }

    pub(super) fn output_to_dataframe(mut self) -> arrow2::error::Result<()> {
        let file = File::options()
            .write(true)
            .truncate(true)
            .open(&self.output)?;
        let writer = BufWriter::new(file);

        let expected_fields = vec![
            Field::new("id", DataType::UInt64, false),
            Field::new("x", DataType::UInt32, false),
            Field::new("y", DataType::UInt32, false),
            Field::new("i", DataType::UInt32, false),
            Field::new("parent", DataType::UInt64, false),
            Field::new("species", DataType::FixedSizeBinary(24), true),
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

        let mut ids = Vec::with_capacity(self.origins.len());
        let mut xs = Vec::with_capacity(self.origins.len());
        let mut ys = Vec::with_capacity(self.origins.len());
        let mut is = Vec::with_capacity(self.origins.len());
        let mut parents = Vec::with_capacity(self.origins.len());

        for (lineage, origin) in &self.origins {
            ids.push(unsafe { lineage.clone().into_inner().get() - 2 });

            xs.push(origin.location().x());
            ys.push(origin.location().y());
            is.push(origin.index());

            parents.push(unsafe {
                self.parents
                    .get(lineage)
                    .unwrap_or(lineage)
                    .clone()
                    .into_inner()
                    .get()
                    - 2
            });
        }

        let mut species = Vec::with_capacity(self.origins.len() * 24);
        let mut has_speciated = MutableBitmap::from_len_zeroed(self.origins.len());

        // Lineage ancestor union-find with path compression
        let mut family = Vec::new();

        for (i, lineage) in self.origins.keys().enumerate() {
            // Find the ancestor that originated the species
            let mut ancestor = lineage.clone();
            while let Some(ancestor_parent) = self.parents.get(&ancestor) {
                family.push(ancestor.clone());
                ancestor = ancestor_parent.clone();
            }

            // Compress the ancestry paths for all visited lineages
            for child in family.drain(..) {
                self.parents.insert(child, ancestor.clone());
            }

            if let Some(identity) = self.species.get(&ancestor) {
                species.extend_from_slice(&**identity);
                has_speciated.set(i, true);
            } else {
                species.extend_from_slice(&[0; 24]);
            }
        }

        let ids = PrimitiveArray::from_vec(ids);
        let xs = PrimitiveArray::from_vec(xs);
        let ys = PrimitiveArray::from_vec(ys);
        let is = PrimitiveArray::from_vec(is);
        let parents = PrimitiveArray::from_vec(parents);
        let species = FixedSizeBinaryArray::try_new(
            DataType::FixedSizeBinary(24),
            Buffer::from(species),
            Some(has_speciated.into()),
        )?;

        let chunk = Chunk::try_new(vec![
            ids.arced(),
            xs.arced(),
            ys.arced(),
            is.arced(),
            parents.arced(),
            species.arced(),
        ])?;
        writer.write(&chunk, None)?;

        writer.finish()
    }
}
