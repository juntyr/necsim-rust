use anyhow::Result;
use log::LevelFilter;

use necsim_core::{event::TypedEvent, reporter::Reporter};

use necsim_partitioning_core::Partitioning;
use necsim_plugins_core::match_any_reporter_plugin_vec;

use crate::args::{CommandArgs, ReplayArgs};

#[allow(clippy::module_name_repetitions, clippy::needless_pass_by_value)]
pub fn replay_with_logger<P: Partitioning>(
    replay_args: CommandArgs,
    partitioning: P,
) -> Result<()> {
    log::set_max_level(LevelFilter::Info);

    let replay_args = ReplayArgs::try_parse(replay_args, &partitioning)?;
    info!("Parsed replay arguments:\n{:#?}", replay_args);

    info!("Starting event replay ...");

    match_any_reporter_plugin_vec!(replay_args.reporters => |mut reporter| {
        reporter.initialise().map_err(anyhow::Error::msg)?;

        let mut remaining = replay_args.log.length() as u64;

        reporter.report_progress(&remaining.into());

        for event in replay_args.log {
            remaining -= 1;
            reporter.report_progress(&remaining.into());

            match event.into() {
                TypedEvent::Speciation(event) => {
                    reporter.report_speciation(&event.into());
                },
                TypedEvent::Dispersal(event) => {
                    reporter.report_dispersal(&event.into());
                },
            }
        }

        if log::log_enabled!(log::Level::Info) {
            println!("\n");
            println!("{:=^80}", " Reporter Summary ");
            println!();
        }
        reporter.finalise();
        if log::log_enabled!(log::Level::Info) {
            println!();
            println!("{:=^80}", " Reporter Summary ");
            println!();
        }
    });

    info!("The event replay has completed.");

    Ok(())
}
