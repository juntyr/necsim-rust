use std::{collections::BinaryHeap, path::PathBuf};

use anyhow::Result;

use necsim_core::event::Event;

mod segment;
mod sorted_segments;

use segment::SortedSegment;
use sorted_segments::SortedSortedSegments;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct EventLogReplay {
    frontier: BinaryHeap<SortedSortedSegments>,
}

impl EventLogReplay {
    /// # Errors
    ///
    /// Returns `Err` iff any of the paths could not be read.
    pub fn try_new(paths: &[PathBuf], capacity: usize) -> Result<Self> {
        let mut segments = Vec::with_capacity(paths.len());

        for path in paths {
            segments.push(SortedSegment::new(path, capacity)?);
        }

        let mut grouped_segments: Vec<Vec<SortedSegment>> = Vec::new();
        let mut current_group: Vec<SortedSegment> = Vec::new();

        let mut max_time: f64 = f64::NEG_INFINITY;

        while !segments.is_empty() {
            let mut min_time: f64 = f64::INFINITY;
            let mut min_index: Option<usize> = None;

            for (i, seg) in segments.iter().enumerate() {
                if seg.header().min_time() > max_time && seg.header().min_time() < min_time {
                    min_time = seg.header().min_time();
                    min_index = Some(i);
                }
            }

            let min_index = if let Some(min_index) = min_index {
                min_index
            } else {
                if !current_group.is_empty() {
                    grouped_segments.push(current_group);
                    current_group = Vec::new();
                }

                max_time = f64::NEG_INFINITY;

                continue;
            };

            let min_segement = segments.swap_remove(min_index);

            max_time = min_segement.header().max_time();

            current_group.push(min_segement);
        }

        if !current_group.is_empty() {
            grouped_segments.push(current_group);
        }

        let mut frontier = BinaryHeap::with_capacity(grouped_segments.len());

        for group in grouped_segments {
            frontier.push(SortedSortedSegments::new(group))
        }

        Ok(Self { frontier })
    }
}

impl Iterator for EventLogReplay {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_segment = self.frontier.pop()?;

        let next_event = next_segment.next();

        self.frontier.push(next_segment);

        next_event
    }
}
