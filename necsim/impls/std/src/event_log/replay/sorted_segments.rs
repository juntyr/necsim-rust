use std::cmp::{Ord, Ordering};

use necsim_core::event::Event;

use super::segment::SortedSegment;

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct SortedSortedSegments {
    segments: Vec<SortedSegment>,
    next: Option<Event>,
}

impl SortedSortedSegments {
    pub fn new(mut segments: Vec<SortedSegment>) -> Self {
        segments.reverse();

        let mut this = Self {
            segments,
            next: None,
        };

        this.next();

        this
    }
}

impl Iterator for SortedSortedSegments {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let next = std::mem::replace(
            &mut self.next,
            loop {
                let next_segment = match self.segments.last_mut() {
                    Some(next_segment) => next_segment,
                    None => break None,
                };

                if let Some(next_event) = next_segment.next() {
                    break Some(next_event);
                }

                self.segments.pop();
            },
        );

        next
    }
}

impl Ord for SortedSortedSegments {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.next, &other.next) {
            (None, None) => Ordering::Equal,
            (None, _) => Ordering::Less,
            (_, None) => Ordering::Greater,
            (Some(this_event), Some(other_event)) => other_event.cmp(this_event),
        }
    }
}

impl PartialOrd for SortedSortedSegments {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SortedSortedSegments {
    fn eq(&self, other: &Self) -> bool {
        self.next.eq(&other.next)
    }
}

impl Eq for SortedSortedSegments {}
