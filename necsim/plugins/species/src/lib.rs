#![deny(clippy::pedantic)]

#[macro_use]
extern crate log;

mod individuals;

// Register the reporter plugin
necsim_plugins_core::export_plugin!(IndividualSpecies => individuals::IndividualLocationSpeciesReporter);
