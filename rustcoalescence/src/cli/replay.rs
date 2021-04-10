use anyhow::Result;
use log::LevelFilter;

use necsim_core::{
    event::TypedEvent,
    reporter::{used::Unused, Reporter},
};

use necsim_plugins_core::match_any_reporter_plugin_vec;

use crate::args::{CommandArgs, ReplayArgs};

#[allow(clippy::module_name_repetitions, clippy::needless_pass_by_value)]
pub fn replay_with_logger(replay_args: CommandArgs) -> Result<()> {
    log::set_max_level(LevelFilter::Info);

    let replay_args = ReplayArgs::try_parse(replay_args)?;
    info!("Parsed replay arguments:\n{:#?}", replay_args);

    info!("Starting event replay ...");

    match_any_reporter_plugin_vec!(replay_args.reporters => |mut reporter| {
        reporter.initialise().map_err(anyhow::Error::msg)?;

        let mut remaining = replay_args.log.length() as u64;

        reporter.report_progress(Unused::new(&remaining));

        for event in replay_args.log {
            remaining -= 1;
            reporter.report_progress(Unused::new(&remaining));

            match event.into() {
                TypedEvent::Speciation(event) => {
                    reporter.report_speciation(Unused::new(&event));
                },
                TypedEvent::Dispersal(event) => {
                    reporter.report_dispersal(Unused::new(&event));
                },
            }
        }

        if log::log_enabled!(log::Level::Info) {
            eprintln!("\n");
            eprintln!("{:=^80}", " Reporter Summary ");
            eprintln!();
        }
        reporter.finalise();
        if log::log_enabled!(log::Level::Info) {
            eprintln!();
            eprintln!("{:=^80}", " Reporter Summary ");
            eprintln!();
        }
    });

    info!("The event replay has completed.");

    Ok(())
}
