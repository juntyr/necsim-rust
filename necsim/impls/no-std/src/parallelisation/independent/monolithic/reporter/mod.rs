use core::ops::ControlFlow;

use necsim_core_bond::NonNegativeF64;

use necsim_core::reporter::{
    boolean::{Boolean, False},
    Reporter,
};

use necsim_partitioning_core::LocalPartition;

mod live;
mod recorded;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[allow(clippy::no_effect_underscore_binding)]
#[contract_trait]
pub trait WaterLevelReporterProxy<'l, 'p, R: Reporter, P: LocalPartition<'p, R>>:
    Sized
    + Reporter<
        ReportSpeciation = R::ReportSpeciation,
        ReportDispersal = R::ReportDispersal,
        ReportProgress = False,
    >
{
    fn new(capacity: usize, local_partition: &'l mut P, sort_batch_size: usize) -> Self;

    fn water_level(&self) -> NonNegativeF64;

    fn partial_sort_step(&mut self) -> ControlFlow<()>;

    #[debug_requires(water_level >= self.water_level(), "advances the water level")]
    #[debug_ensures(self.water_level() == old(water_level))]
    fn advance_water_level(&mut self, water_level: NonNegativeF64);

    fn local_partition(&mut self) -> &mut P;
}

#[allow(clippy::empty_enum)]
pub enum WaterLevelReporterStrategy {}

pub trait WaterLevelReporterConstructor<
    'l,
    'p,
    IsLive: Boolean,
    R: Reporter,
    P: 'l + LocalPartition<'p, R, IsLive = IsLive>,
>
{
    type WaterLevelReporter: WaterLevelReporterProxy<'l, 'p, R, P>;
}

impl<'l, 'p, IsLive: Boolean, R: Reporter, P: 'l + LocalPartition<'p, R, IsLive = IsLive>>
    WaterLevelReporterConstructor<'l, 'p, IsLive, R, P> for WaterLevelReporterStrategy
{
    default type WaterLevelReporter = live::LiveWaterLevelReporterProxy<'l, 'p, R, P>;
}

impl<'l, 'p, R: Reporter, P: 'l + LocalPartition<'p, R, IsLive = False>>
    WaterLevelReporterConstructor<'l, 'p, False, R, P> for WaterLevelReporterStrategy
{
    type WaterLevelReporter = recorded::RecordedWaterLevelReporterProxy<'l, 'p, R, P>;
}
