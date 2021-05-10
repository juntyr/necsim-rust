use necsim_core_bond::NonNegativeF64;

use necsim_core::reporter::{
    boolean::{Boolean, False},
    Reporter,
};

use crate::partitioning::LocalPartition;

mod live;
mod recorded;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait WaterLevelReporterProxy<'p, R: Reporter, P: LocalPartition<R>>:
    Sized
    + Reporter<
        ReportSpeciation = R::ReportSpeciation,
        ReportDispersal = R::ReportDispersal,
        ReportProgress = False,
    >
{
    fn new(capacity: usize, local_partition: &'p mut P) -> Self;

    fn water_level(&self) -> NonNegativeF64;

    #[debug_requires(water_level >= self.water_level(), "advances the water level")]
    #[debug_ensures(self.water_level() == old(water_level))]
    fn advance_water_level(&mut self, water_level: NonNegativeF64);

    fn local_partition(&mut self) -> &mut P;
}

#[allow(clippy::empty_enum)]
pub enum WaterLevelReporterStrategy {}

pub trait WaterLevelReporterConstructor<
    'p,
    IsLive: Boolean,
    R: Reporter,
    P: 'p + LocalPartition<R, IsLive = IsLive>,
>
{
    type WaterLevelReporter: WaterLevelReporterProxy<'p, R, P>;
}

impl<'p, IsLive: Boolean, R: Reporter, P: 'p + LocalPartition<R, IsLive = IsLive>>
    WaterLevelReporterConstructor<'p, IsLive, R, P> for WaterLevelReporterStrategy
{
    default type WaterLevelReporter = live::LiveWaterLevelReporterProxy<'p, R, P>;
}

impl<'p, R: Reporter, P: 'p + LocalPartition<R, IsLive = False>>
    WaterLevelReporterConstructor<'p, False, R, P> for WaterLevelReporterStrategy
{
    type WaterLevelReporter = recorded::RecordedWaterLevelReporterProxy<'p, R, P>;
}
