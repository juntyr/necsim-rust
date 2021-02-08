use anyhow::Result;
use hashbrown::{hash_map::RawEntryMut, HashMap};

use necsim_core::{
    event::{Event, EventType},
    reporter::{EventFilter, Reporter},
};

use necsim_impls_no_std::reporter::ReporterContext;
use necsim_impls_std::event_replay::{EventReplayIterator, EventReplayType};

use crate::args::ReplayArgs;

#[allow(clippy::module_name_repetitions)]
pub fn replay_with_logger<R: ReporterContext>(
    replay_args: &ReplayArgs,
    reporter_context: R,
) -> Result<()> {
    anyhow::ensure!(
        !replay_args.events().is_empty(),
        "The replay command must be given at least one event log."
    );

    let mut remaining = 0_u64;
    let mut any_overlapping = false;

    for path in replay_args.events() {
        anyhow::ensure!(
            path.exists(),
            format!("The event log {:?} does not exist.", path)
        );

        remaining += match EventReplayIterator::try_new(path)? {
            EventReplayType::Disjoint(iter) => iter.len(),
            EventReplayType::Overlapping(iter) => {
                any_overlapping = true;

                iter.len()
            },
        };
    }

    let progress_update_step = remaining / 100;

    #[allow(clippy::cast_possible_truncation)]
    let mut event_deduplicator: HashMap<Event, ()> = if any_overlapping {
        HashMap::with_capacity((remaining as usize) / replay_args.events().len())
    } else {
        HashMap::new()
    };

    info!("Starting event replay ...");

    let mut reporter = reporter_context.build_guarded();
    let mut completed = 0_u64;

    for path in replay_args.events() {
        match EventReplayIterator::try_new(path)? {
            EventReplayType::Disjoint(iter) => iter.for_each(|event| {
                reporter.report_event(&event);

                completed += 1;
                if completed % progress_update_step == 0 {
                    reporter.report_progress(remaining - completed);
                }
            }),
            EventReplayType::Overlapping(iter) => iter.for_each(|event| {
                if (R::Reporter::REPORT_SPECIATION
                    && matches!(event.r#type(), EventType::Speciation))
                    || (R::Reporter::REPORT_DISPERSAL
                        && matches!(event.r#type(), EventType::Dispersal { .. }))
                {
                    if let RawEntryMut::Vacant(entry) =
                        event_deduplicator.raw_entry_mut().from_key(&event)
                    {
                        reporter.report_event(entry.insert(event, ()).0)
                    }

                    completed += 1;
                    if completed % progress_update_step == 0 {
                        reporter.report_progress(remaining - completed);
                    }
                }
            }),
        }
    }

    reporter.report_progress(0);

    std::mem::drop(reporter);

    info!("The event replay has completed.");

    Ok(())
}
