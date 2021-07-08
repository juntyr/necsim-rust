#![deny(clippy::pedantic)]

mod provenance;
mod reporter;

// Register the reporter plugin
necsim_plugins_core::export_plugin!(Tree => reporter::TskitTreeReporter);
