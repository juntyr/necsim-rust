use std::{collections::BTreeMap, fs::File, io::BufWriter};

use arrow2::{
    array::{FixedSizeBinaryArray, PrimitiveArray},
    chunk::Chunk,
    datatypes::{DataType, Field, Schema},
    io::ipc::write::{FileWriter, WriteOptions},
};
use necsim_core::{landscape::IndexedLocation, lineage::GlobalLineageReference};
use necsim_core_bond::PositiveF64;

use super::{IndividualLocationSpeciesReporter, LastEventState, SpeciesIdentity};

impl IndividualLocationSpeciesReporter {
    pub(super) fn store_individual_origin(
        &mut self,
        individual: &GlobalLineageReference,
        origin: &IndexedLocation,
    ) {
        self.origins.insert(individual.clone(), origin.clone());
    }

    pub(super) fn store_individual_speciation(
        &mut self,
        parent: &GlobalLineageReference,
        origin: &IndexedLocation,
        time: PositiveF64,
    ) {
        // Resolve the actual parent, irrespective of duplicate individuals
        let mut parent = parent;
        while let Some(parent_parent) = self.parents.get(parent) {
            parent = parent_parent;
        }
        let parent = parent.clone();

        let location = (u64::from(origin.location().y()) << 32) | u64::from(origin.location().x());
        let index = u64::from(origin.index()) << 16;
        let time = time.get().to_bits();

        let location_bytes = seahash_diffuse(location).to_le_bytes();
        let index_bytes = seahash_diffuse(index).to_le_bytes();
        let time_bytes = seahash_diffuse(time).to_le_bytes();

        // Shuffle and mix all 24 bytes of the species identity
        let lower = seahash_diffuse(u64::from_le_bytes([
            location_bytes[3],
            time_bytes[0],
            index_bytes[5],
            location_bytes[1],
            time_bytes[4],
            time_bytes[7],
            time_bytes[5],
            location_bytes[5],
        ]))
        .to_le_bytes();
        let middle = seahash_diffuse(u64::from_le_bytes([
            time_bytes[6],
            index_bytes[4],
            location_bytes[0],
            location_bytes[6],
            index_bytes[2],
            index_bytes[1],
            location_bytes[7],
            index_bytes[3],
        ]))
        .to_le_bytes();
        let upper = seahash_diffuse(u64::from_le_bytes([
            location_bytes[4],
            location_bytes[2],
            time_bytes[2],
            index_bytes[0],
            time_bytes[3],
            time_bytes[1],
            index_bytes[7],
            index_bytes[6],
        ]))
        .to_le_bytes();

        self.species.insert(
            parent,
            SpeciesIdentity([
                lower[0], lower[1], lower[2], lower[3], lower[4], lower[5], lower[6], lower[7],
                middle[0], middle[1], middle[2], middle[3], middle[4], middle[5], middle[6],
                middle[7], upper[0], upper[1], upper[2], upper[3], upper[4], upper[5], upper[6],
                upper[7],
            ]),
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
            Field::new("parent", DataType::UInt64, true),
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
        let mut species = Vec::with_capacity(self.origins.len());

        // Lineage ancestor union-find with path compression
        let mut ancestors = self.parents.clone();

        let mut family = Vec::new();

        for (lineage, origin) in std::mem::take(&mut self.origins) {
            // Find the ancestor that originated the species
            let mut ancestor = lineage.clone();
            while let Some(ancestor_parent) = ancestors.get(&ancestor) {
                family.push(ancestor.clone());
                ancestor = ancestor_parent.clone();
            }

            // Compress the ancestry paths for all visited lineages
            for child in family.drain(..) {
                ancestors.insert(child, ancestor.clone());
            }

            ids.push(unsafe { lineage.clone().into_inner().get() - 2 });
            xs.push(origin.location().x());
            ys.push(origin.location().y());
            is.push(origin.index());
            parents.push(
                self.parents
                    .get(&lineage)
                    .map(|parent| unsafe { parent.clone().into_inner().get() - 2 }),
            );
            species.push(self.species.get(&ancestor).map(|species| species.0));
        }

        let ids = PrimitiveArray::from_vec(ids);
        let xs = PrimitiveArray::from_vec(xs);
        let ys = PrimitiveArray::from_vec(ys);
        let is = PrimitiveArray::from_vec(is);
        let parents = PrimitiveArray::from_trusted_len_iter(parents.into_iter());
        let species = FixedSizeBinaryArray::from_iter(species.into_iter(), 24);

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
