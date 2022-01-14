use necsim_core::{
    event::{DispersalEvent, SpeciationEvent},
    landscape::{IndexedLocation, Location},
    lineage::GlobalLineageReference,
};
use necsim_core_bond::{NonNegativeF64, NonZeroOneU64, PositiveF64};

use bincode::Options;
use rusqlite::{named_params, types::Value};
use serde::{Deserialize, Serialize};

use super::{SpeciesIdentity, SpeciesLocationsMode, SpeciesLocationsReporter};

const METADATA_TABLE: &str = "__SPECIES_REPORTER_META";

impl SpeciesLocationsReporter {
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

        // Prevent a lookup-loop, can occur after `Resume`
        if child != parent {
            self.parents.insert(child, parent);
        }
    }

    #[allow(clippy::too_many_lines)]
    pub(super) fn initialise_sqlite_connection(&mut self) -> rusqlite::Result<()> {
        // Create the species locations table in `Create` mode
        if let SpeciesLocationsMode::Create = self.mode {
            self.connection
                .execute_batch(&format!(
                    "CREATE TABLE {} (
                            id      INTEGER PRIMARY KEY NOT NULL,
                            x       INTEGER NOT NULL,
                            y       INTEGER NOT NULL,
                            i       INTEGER NOT NULL,
                            parent  INTEGER,
                            species TEXT
                        );
                        CREATE TABLE {} (
                            key     TEXT PRIMARY KEY NOT NULL,
                            value   TEXT NOT NULL
                        );",
                    self.table, METADATA_TABLE,
                ))
                .map(|_| ())?;
        }

        let mut schema: Vec<Vec<Value>> = Vec::new();

        // Collect the schema information for the species locations table
        self.connection
            .pragma(None, "table_info", &self.table, |row| {
                let mut schema_row = Vec::new();

                for col in 0..row.as_ref().column_count() {
                    schema_row.push(Value::from(row.get_ref(col)?));
                }

                schema.push(schema_row);

                Ok(())
            })?;

        // Check that the schema of the species locations table matches
        if schema
            != vec![
                vec![
                    Value::Integer(0),
                    Value::Text(String::from("id")),
                    Value::Text(String::from("INTEGER")),
                    Value::Integer(1),
                    Value::Null,
                    Value::Integer(1),
                ],
                vec![
                    Value::Integer(1),
                    Value::Text(String::from("x")),
                    Value::Text(String::from("INTEGER")),
                    Value::Integer(1),
                    Value::Null,
                    Value::Integer(0),
                ],
                vec![
                    Value::Integer(2),
                    Value::Text(String::from("y")),
                    Value::Text(String::from("INTEGER")),
                    Value::Integer(1),
                    Value::Null,
                    Value::Integer(0),
                ],
                vec![
                    Value::Integer(3),
                    Value::Text(String::from("i")),
                    Value::Text(String::from("INTEGER")),
                    Value::Integer(1),
                    Value::Null,
                    Value::Integer(0),
                ],
                vec![
                    Value::Integer(4),
                    Value::Text(String::from("parent")),
                    Value::Text(String::from("INTEGER")),
                    Value::Integer(0),
                    Value::Null,
                    Value::Integer(0),
                ],
                vec![
                    Value::Integer(5),
                    Value::Text(String::from("species")),
                    Value::Text(String::from("TEXT")),
                    Value::Integer(0),
                    Value::Null,
                    Value::Integer(0),
                ],
            ]
        {
            return Err(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::SchemaChanged,
                    extended_code: 0,
                },
                Some(format!(
                    "Invalid schema for the species locations table {}",
                    self.table
                )),
            ));
        }

        let mut schema: Vec<Vec<Value>> = Vec::new();

        // Collect the schema information for the metadata table
        self.connection
            .pragma(None, "table_info", METADATA_TABLE, |row| {
                let mut schema_row = Vec::new();

                for col in 0..row.as_ref().column_count() {
                    schema_row.push(Value::from(row.get_ref(col)?));
                }

                schema.push(schema_row);

                Ok(())
            })?;

        // Check that the schema of the metadata table matches
        if schema
            != vec![
                vec![
                    Value::Integer(0),
                    Value::Text(String::from("key")),
                    Value::Text(String::from("TEXT")),
                    Value::Integer(1),
                    Value::Null,
                    Value::Integer(1),
                ],
                vec![
                    Value::Integer(1),
                    Value::Text(String::from("value")),
                    Value::Text(String::from("TEXT")),
                    Value::Integer(1),
                    Value::Null,
                    Value::Integer(0),
                ],
            ]
        {
            return Err(rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::SchemaChanged,
                    extended_code: 0,
                },
                Some(String::from(
                    "Invalid schema for the internal metadata table",
                )),
            ));
        }

        // Early return for the `Create` mode
        if let SpeciesLocationsMode::Create = self.mode {
            return Ok(());
        }

        let mut statement = self.connection.prepare(&format!(
            "SELECT id, x, y, i, parent, species FROM {}",
            self.table
        ))?;
        let mut query = statement.query([])?;

        // Resume from the existing species locations in the table
        while let Some(row) = query.next()? {
            let id: u64 = row.get("id")?;
            let x: u32 = row.get("x")?;
            let y: u32 = row.get("y")?;
            let i: u32 = row.get("i")?;

            let parent: Option<u64> = row.get("parent")?;
            let species: Option<String> = row.get("species")?;

            let id =
                unsafe { GlobalLineageReference::from_inner(NonZeroOneU64::new_unchecked(id + 2)) };

            // Populate the individual `origins` lookup
            self.origins
                .insert(id.clone(), IndexedLocation::new(Location::new(x, y), i));

            if let Some(parent) = parent {
                let parent = unsafe {
                    GlobalLineageReference::from_inner(NonZeroOneU64::new_unchecked(parent + 2))
                };

                // Populate the individual `parents` lookup
                self.parents.insert(id.clone(), parent);
            }

            if let Some(species) = species {
                // Try to parse the species identity from its String form
                let species = (|| -> Result<_, _> {
                    if species.len() != 48 || !species.is_ascii() {
                        return Err(());
                    }

                    Ok(SpeciesIdentity(
                        u64::from_str_radix(&species[0..16], 16).map_err(|_| ())?,
                        u64::from_str_radix(&species[16..32], 16).map_err(|_| ())?,
                        u64::from_str_radix(&species[32..48], 16).map_err(|_| ())?,
                    ))
                })()
                .map_err(|_| {
                    rusqlite::Error::SqliteFailure(
                        rusqlite::ffi::Error {
                            code: rusqlite::ffi::ErrorCode::TypeMismatch,
                            extended_code: 0,
                        },
                        Some(format!(
                            "Invalid species identity {:?} for individual #{}",
                            species, id,
                        )),
                    )
                })?;

                // Populate the individual `species` lookup
                self.species.insert(id, species);
            }
        }

        let last_event: String = self
            .connection
            .query_row(
                &format!(
                    "SELECT value FROM {} WHERE key='last-event'",
                    METADATA_TABLE
                ),
                [],
                |row| row.get("value"),
            )
            .map_err(|_| {
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error {
                        code: rusqlite::ffi::ErrorCode::NotFound,
                        extended_code: 0,
                    },
                    Some(String::from("Failed to fetch the reporter resume metadata")),
                )
            })?;

        let LastEventState {
            last_parent_prior_time,
            last_speciation_event,
            last_dispersal_event,
        } = LastEventState::from_string(&last_event).map_err(|_| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::TypeMismatch,
                    extended_code: 0,
                },
                Some(String::from(
                    "Failed to decode the reporter resume metadata",
                )),
            )
        })?;

        self.last_parent_prior_time = last_parent_prior_time;
        self.last_speciation_event = last_speciation_event;
        self.last_dispersal_event = last_dispersal_event;

        Ok(())
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

            insertion.execute(named_params! {
                ":id": lineage.to_string(),
                ":x": origin.location().x(),
                ":y": origin.location().y(),
                ":i": origin.index(),
                ":parent": self.parents.get(&lineage).map(|parent| format!("{}", parent)),
                ":species": self.species.get(ancestor).map(|species| {
                    format!("{:016x}{:016x}{:016x}", species.0, species.1, species.2)
                }),
            })?;
        }

        insertion.finalize()?;

        let last_event_state = LastEventState {
            last_parent_prior_time: self.last_parent_prior_time,
            last_speciation_event: self.last_speciation_event,
            last_dispersal_event: self.last_dispersal_event,
        }
        .into_string()
        .map_err(|_| {
            rusqlite::Error::SqliteFailure(
                rusqlite::ffi::Error {
                    code: rusqlite::ffi::ErrorCode::TypeMismatch,
                    extended_code: 0,
                },
                Some(String::from(
                    "Failed to encode the reporter resume metadata",
                )),
            )
        })?;

        tx.execute(
            &format!(
                "INSERT OR REPLACE INTO {} VALUES (:key, :value)",
                METADATA_TABLE
            ),
            named_params! { ":key": "last-event", ":value": last_event_state },
        )?;

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
