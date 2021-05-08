use serde::Deserialize;
use std::{collections::BinaryHeap, convert::TryFrom, iter::FromIterator};

use necsim_core::event::PackedEvent;

mod globbed;
pub mod segment;
mod sorted_segments;

use globbed::GlobbedSortedSegments;
use segment::SortedSegment;
use sorted_segments::SortedSortedSegments;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Deserialize)]
#[serde(try_from = "Vec<GlobbedSortedSegments>")]
pub struct EventLogReplay {
    frontier: BinaryHeap<SortedSortedSegments>,

    with_speciation: bool,
    with_dispersal: bool,
}

impl TryFrom<Vec<GlobbedSortedSegments>> for EventLogReplay {
    type Error = anyhow::Error;

    fn try_from(vec: Vec<GlobbedSortedSegments>) -> Result<Self, Self::Error> {
        vec.into_iter().flatten().collect()
    }
}

impl EventLogReplay {
    #[must_use]
    pub fn length(&self) -> usize {
        self.frontier.iter().map(SortedSortedSegments::length).sum()
    }

    #[must_use]
    pub fn with_speciation(&self) -> bool {
        self.with_speciation
    }

    #[must_use]
    pub fn with_dispersal(&self) -> bool {
        self.with_dispersal
    }
}

impl FromIterator<SortedSegment> for anyhow::Result<EventLogReplay> {
    fn from_iter<T: IntoIterator<Item = SortedSegment>>(iter: T) -> Self {
        let mut segments: Vec<SortedSegment> = iter.into_iter().collect();

        if segments.is_empty() {
            anyhow::bail!("The EventLogReplay requires at least one event log segment.")
        }

        let mut with_speciation = None;
        let mut with_dispersal = None;

        for segment in &segments {
            if let Some(with_speciation) = with_speciation {
                anyhow::ensure!(
                    with_speciation == segment.header().with_speciation(),
                    "There is a mismatch in reporting speciation events between some segments."
                );
            } else {
                with_speciation = Some(segment.header().with_speciation());
            }

            if let Some(with_dispersal) = with_dispersal {
                anyhow::ensure!(
                    with_dispersal == segment.header().with_dispersal(),
                    "There is a mismatch in reporting dispersal events between some segments."
                );
            } else {
                with_dispersal = Some(segment.header().with_dispersal());
            }
        }

        let mut grouped_segments: Vec<Vec<SortedSegment>> = Vec::new();
        let mut current_group: Vec<SortedSegment> = Vec::new();

        let mut max_time: f64 = f64::NEG_INFINITY;

        while !segments.is_empty() {
            let mut min_time: f64 = f64::INFINITY;
            let mut min_index: Option<usize> = None;

            for (i, seg) in segments.iter().enumerate() {
                if seg.header().min_time() > max_time && seg.header().min_time() < min_time {
                    min_time = seg.header().min_time().get();
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

            max_time = min_segement.header().max_time().get();

            current_group.push(min_segement);
        }

        if !current_group.is_empty() {
            grouped_segments.push(current_group);
        }

        let mut frontier = BinaryHeap::with_capacity(grouped_segments.len());

        for group in grouped_segments {
            frontier.push(SortedSortedSegments::new(group))
        }

        Ok(EventLogReplay {
            frontier,
            with_speciation: with_speciation.unwrap(),
            with_dispersal: with_dispersal.unwrap(),
        })
    }
}

impl Iterator for EventLogReplay {
    type Item = PackedEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_segment = self.frontier.pop()?;

        let next_event = next_segment.next();

        self.frontier.push(next_segment);

        next_event
    }
}
