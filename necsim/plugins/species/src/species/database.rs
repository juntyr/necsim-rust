use necsim_core::{landscape::IndexedLocation, lineage::GlobalLineageReference};
use necsim_core_bond::PositiveF64;

use rusqlite::named_params;

use super::{SpeciesIdentity, SpeciesLocationsReporter};

impl SpeciesLocationsReporter {
    pub(super) fn store_individual_origin(
        &mut self,
        individual: &GlobalLineageReference,
        origin: &IndexedLocation,
    ) {
        self.origins.push((individual.clone(), origin.clone()));
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

        self.species.insert(
            parent,
            SpeciesIdentity(
                seahash_diffuse(location),
                seahash_diffuse(index),
                seahash_diffuse(time),
            ),
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

        self.parents.insert(child, parent);
    }

    pub(super) fn initialise_sqlite_connection(&mut self) -> rusqlite::Result<()> {
        self.connection
            .execute(
                &format!(
                    "CREATE TABLE IF NOT EXISTS {} (
                    id      INTEGER PRIMARY KEY NOT NULL,
                    x       INTEGER NOT NULL,
                    y       INTEGER NOT NULL,
                    i       INTEGER NOT NULL,
                    parent  INTEGER NOT NULL,
                    species TEXT NOT NULL
                )",
                    self.table
                ),
                [],
            )
            .map(|_| ())
    }

    pub(super) fn output_to_database(mut self) -> rusqlite::Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(rusqlite::TransactionBehavior::Exclusive)?;

        let mut insertion = tx.prepare(&format!(
            "INSERT OR REPLACE INTO {} VALUES (:id, :x, :y, :i, :parent, :species)",
            self.table,
        ))?;

        for (lineage, origin) in self.origins {
            let mut ancestor = &lineage;
            while let Some(ancestor_parent) = self.parents.get(ancestor) {
                ancestor = ancestor_parent;
            }

            let species = match self.species.get(ancestor) {
                Some(species) => species,
                None => continue,
            };

            insertion.execute(named_params! {
                ":id": lineage.to_string(),
                ":x": origin.location().x(),
                ":y": origin.location().y(),
                ":i": origin.index(),
                ":parent": self.parents.get(&lineage).unwrap_or(&lineage).to_string(),
                ":species": format!("{:016x}{:016x}{:016x}", species.0, species.1, species.2),
            })?;
        }

        insertion.finalize()?;

        tx.commit()?;

        self.connection.close().map_err(|(_, err)| err)
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
