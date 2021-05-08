use std::cmp::Ordering;

use necsim_core_bond::{NonNegativeF64, PositiveF64};

#[derive(PartialEq, Copy, Clone)]
pub struct EventTime(NonNegativeF64);

impl Eq for EventTime {}

impl PartialOrd for EventTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl Ord for EventTime {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

impl From<NonNegativeF64> for EventTime {
    fn from(event_time: NonNegativeF64) -> Self {
        Self(event_time)
    }
}

impl From<PositiveF64> for EventTime {
    fn from(event_time: PositiveF64) -> Self {
        Self(event_time.into())
    }
}

impl From<EventTime> for NonNegativeF64 {
    fn from(event_time: EventTime) -> Self {
        event_time.0
    }
}
