#![deny(clippy::pedantic)]

mod provenance;
mod reporter;

necsim_plugins_core::export_plugin!(Tree => reporter::TskitTreeReporter);
