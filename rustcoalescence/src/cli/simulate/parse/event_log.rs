use anyhow::Context;
use serde::{Deserialize, Deserializer};

use necsim_impls_std::event_log::recorder::EventLogRecorder;

use crate::args::{parse::try_parse_state, Partitioning};

use super::super::BufferingSimulateArgsBuilder;

pub(in super::super) fn parse_and_normalise(
    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
    partitioning: &Partitioning,
) -> anyhow::Result<Option<EventLogRecorder>> {
    let mut event_log_check = partitioning.get_event_log_check();

    let SimulateArgsEventLogOnly { event_log } =
        try_parse_state("simulate", ron_args, &mut event_log_check)?;

    match &event_log {
        None => event_log_check.0,
        Some(_) => event_log_check.1,
    }
    .map_err(|err| anyhow::anyhow!("simulate.*: {}", err))
    .context("Failed to parse the simulate subcommand arguments.")?;

    normalised_args.log(&event_log);

    Ok(event_log)
}

#[derive(DeserializeState)]
#[serde(deserialize_state = "(anyhow::Result<()>, anyhow::Result<()>)")]
#[serde(rename = "Simulate")]
struct SimulateArgsEventLogOnly {
    #[serde(alias = "log")]
    #[serde(default)]
    #[serde(deserialize_state_with = "deserialize_state_event_log")]
    event_log: Option<EventLogRecorder>,
}

fn deserialize_state_event_log<'de, D: Deserializer<'de>>(
    event_log_check: &mut (anyhow::Result<()>, anyhow::Result<()>),
    deserializer: D,
) -> Result<Option<EventLogRecorder>, D::Error> {
    let maybe_event_log = <Option<EventLogRecorder>>::deserialize(deserializer)?;

    if maybe_event_log.is_none() {
        event_log_check
            .0
            .as_ref()
            .map_err(serde::de::Error::custom)?;
    } else {
        event_log_check
            .1
            .as_ref()
            .map_err(serde::de::Error::custom)?;
    }

    Ok(maybe_event_log)
}
