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
        crate::f64::total_cmp_f64(other.0, self.0)
    }
}

impl From<f64> for EventTime {
    fn from(event_time: f64) -> Self {
        Self(event_time)
    }
}

impl Into<f64> for EventTime {
    fn into(self) -> f64 {
        self.0
    }
}
