#![deny(clippy::pedantic)]

#[macro_use]
extern crate log;

mod arrow;
mod feather;
mod individuals;

// Register the reporter plugins
necsim_plugins_core::export_plugin!(
    IndividualSpeciesSQL => individuals::IndividualLocationSpeciesReporter,
    IndividualSpeciesArrow => arrow::IndividualLocationSpeciesReporter,
    IndividualSpeciesFeather => feather::LocationGroupedSpeciesReporter,
);
