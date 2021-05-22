#![deny(clippy::pedantic)]

#[macro_use]
extern crate contracts;

#[macro_use]
extern crate log;

pub mod biodiversity;
pub mod event_counter;
pub mod execution_time;
pub mod progress;
pub mod verbose;

necsim_plugins_core::export_plugin!(
    Biodiversity => biodiversity::BiodiversityReporter,
    Progress => progress::ProgressReporter,
    Execution => execution_time::ExecutionTimeReporter,
    Counter => event_counter::EventCounterReporter,
    Verbose => verbose::VerboseReporter,
);
