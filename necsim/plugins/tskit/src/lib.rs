#![deny(clippy::pedantic)]

mod provenance;
mod tree;

// Register the reporter plugin
necsim_plugins_core::export_plugin!(Tree => tree::TskitTreeReporter);
