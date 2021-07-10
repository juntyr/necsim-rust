#![deny(clippy::pedantic)]

#[macro_use]
extern crate log;

mod species;

// Register the reporter plugin
necsim_plugins_core::export_plugin!(Species => species::SpeciesLocationsReporter);
