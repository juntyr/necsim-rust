use std::cmp::Ordering;

#[derive(PartialEq, Copy, Clone)]
pub struct EventTime(f64);

impl Eq for EventTime {}

impl PartialOrd for EventTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl Ord for EventTime {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.total_cmp(&self.0)
    }
}

impl From<f64> for EventTime {
    fn from(event_time: f64) -> Self {
        Self(event_time)
    }
}

impl From<EventTime> for f64 {
    fn from(event_time: EventTime) -> Self {
        event_time.0
    }
}
