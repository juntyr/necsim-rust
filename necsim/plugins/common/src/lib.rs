#![deny(clippy::pedantic)]

necsim_plugins_core::export_plugin!(
    Biodiversity => necsim_impls_std::reporter::biodiversity::BiodiversityReporter,
    Progress => necsim_impls_std::reporter::progress::ProgressReporter,
    Execution => necsim_impls_std::reporter::execution_time::ExecutionTimeReporter,
    Counter => necsim_impls_std::reporter::event_counter::EventCounterReporter,
    Verbose => necsim_impls_std::reporter::verbose::VerboseReporter,
);
