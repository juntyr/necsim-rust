#![deny(clippy::pedantic)]

mod coverage;
mod speciation;
mod turnover;

necsim_plugins_core::export_plugin!(
    GlobalTurnover => turnover::GlobalTurnoverReporter,
    GlobalSpeciation => speciation::GlobalSpeciationReporter,
    GlobalCoverage => coverage::GlobalCoverageReporter,
);
