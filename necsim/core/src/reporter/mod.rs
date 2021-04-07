use crate::event::{DispersalEvent, SpeciationEvent};

mod combinator;
mod group;
mod r#impl;
mod null;

use boolean::Boolean;
use used::{MaybeUsed, Unused};

pub mod boolean;
pub mod used;

pub use combinator::ReporterCombinator;
pub use null::NullReporter;

pub trait Reporter {
    type ReportSpeciation: Boolean;
    type ReportDispersal: Boolean;
    type ReportProgress: Boolean;

    fn report_speciation<'a>(
        &mut self,
        event: Unused<'a, SpeciationEvent>,
    ) -> MaybeUsed<'a, SpeciationEvent, Self::ReportSpeciation>;

    fn report_dispersal<'a>(
        &mut self,
        event: Unused<'a, DispersalEvent>,
    ) -> MaybeUsed<'a, DispersalEvent, Self::ReportDispersal>;

    fn report_progress<'a>(
        &mut self,
        remaining: Unused<'a, u64>,
    ) -> MaybeUsed<'a, u64, Self::ReportProgress>;
}
