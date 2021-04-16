#![deny(clippy::pedantic)]

mod turnover;

necsim_plugins_core::export_plugin!(
    GlobalTurnover => turnover::GlobalTurnoverReporter,
);
