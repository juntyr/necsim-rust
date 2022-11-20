#![deny(clippy::pedantic)]

#[macro_use]
extern crate log;

mod identity;
mod individual;
mod location;
mod state;

use identity::SpeciesIdentity;
use state::LastEventState;

// Register the reporter plugins
necsim_plugins_core::export_plugin!(
    IndividualSpeciesSQLite => individual::sqlite::IndividualSpeciesSQLiteReporter,
    IndividualSpeciesFeather => individual::feather::IndividualSpeciesFeatherReporter,
    LocationSpeciesFeather => location::feather::LocationSpeciesFeatherReporter,
);
