use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

pub mod recorder;
pub mod replay;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[allow(clippy::module_name_repetitions)]
pub struct EventLogHeader {
    min_time: f64,
    max_time: f64,
}

impl EventLogHeader {
    #[must_use]
    pub fn new(min_time: f64, max_time: f64) -> Self {
        Self { min_time, max_time }
    }

    #[must_use]
    pub fn min_time(&self) -> f64 {
        self.min_time
    }

    #[must_use]
    pub fn max_time(&self) -> f64 {
        self.max_time
    }
}

impl Eq for EventLogHeader {}

impl PartialOrd for EventLogHeader {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.max_time < other.min_time {
            Some(Ordering::Less)
        } else if self.min_time > other.max_time {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}
