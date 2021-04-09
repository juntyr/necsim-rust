#![deny(clippy::pedantic)]

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate log;

mod biodiversity;
mod event_counter;
mod execution_time;
mod progress;
mod verbose;

necsim_plugins_core::export_plugin!(
    Biodiversity => biodiversity::BiodiversityReporter,
    Progress => progress::ProgressReporter,
    Execution => execution_time::ExecutionTimeReporter,
    Counter => event_counter::EventCounterReporter,
    Verbose => verbose::VerboseReporter,
);
