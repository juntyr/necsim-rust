#![allow(clippy::module_name_repetitions)]

use super::stages::{
    CoalescenceSampler, ConditionalCoalescenceSampler, ConditionalEvent, ConditionalProbability,
    DispersalSampler, EventStage, Habitat, LineageReference, LineageSampler, ProbabilityStage,
    SeparableDispersalSampler, UnconditionalEvent, UnconditionalProbability,
};

use crate::landscape::{LandscapeExtent, Location};
use crate::lineage::Lineage;
use crate::rng::Rng;

pub struct TestHabitat;
#[contract_trait]
impl Habitat for TestHabitat {
    #[must_use]
    fn get_extent(&self) -> LandscapeExtent {
        unimplemented!()
    }

    #[must_use]
    fn get_total_habitat(&self) -> usize {
        unimplemented!()
    }

    #[must_use]
    fn get_habitat_at_location(&self, location: &Location) -> u32 {
        unimplemented!()
    }
}

pub struct TestBaseDispersalSampler;
pub struct TestSeparableDispersalSampler;

impl<H: Habitat> DispersalSampler<H> for TestBaseDispersalSampler {
    #[must_use]
    fn sample_dispersal_from_location(&self, location: &Location, rng: &mut impl Rng) -> Location {
        unimplemented!()
    }
}
impl<H: Habitat> DispersalSampler<H> for TestSeparableDispersalSampler {
    #[must_use]
    fn sample_dispersal_from_location(&self, location: &Location, rng: &mut impl Rng) -> Location {
        unimplemented!()
    }
}
#[contract_trait]
impl<H: Habitat> SeparableDispersalSampler<H> for TestSeparableDispersalSampler {
    #[must_use]
    fn sample_non_self_dispersal_from_location(
        &self,
        location: &Location,
        rng: &mut impl Rng,
    ) -> Location {
        unimplemented!()
    }

    #[must_use]
    fn get_self_dispersal_probability(&self, location: &Location) -> f64 {
        unimplemented!()
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct TestLineageReference;

impl<H: Habitat> LineageReference<H> for TestLineageReference {}

pub struct TestLineageSampler<H: Habitat>(std::marker::PhantomData<H>);

impl<H: Habitat, R: LineageReference<H>> std::ops::Index<R> for TestLineageSampler<H> {
    type Output = Lineage;

    #[must_use]
    fn index(&self, reference: R) -> &Self::Output {
        unimplemented!()
    }
}
#[contract_trait]
impl<H: Habitat, R: LineageReference<H>> LineageSampler<H, R> for TestLineageSampler<H> {
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        unimplemented!()
    }

    #[must_use]
    fn pop_next_active_lineage_reference_and_event_time(
        &mut self,
        time: f64,
        rng: &mut impl Rng,
    ) -> Option<(R, f64)> {
        unimplemented!()
    }

    fn add_lineage_reference_to_location(
        &mut self,
        reference: R,
        location: Location,
        time: f64,
        rng: &mut impl Rng,
    ) {
        unimplemented!()
    }
}

pub struct TestCoalescenceSampler;
pub struct TestConditionalCoalescenceSampler;

#[contract_trait]
impl<H: Habitat, R: LineageReference<H>, L: LineageSampler<H, R>> CoalescenceSampler<H, R, L>
    for TestCoalescenceSampler
{
    #[must_use]
    fn sample_optional_coalescence_at_location(
        &self,
        location: &Location,
        habitat: u32,
        rng: &mut impl Rng,
    ) -> Option<R> {
        unimplemented!()
    }
}
#[contract_trait]
impl<H: Habitat, R: LineageReference<H>, L: LineageSampler<H, R>> CoalescenceSampler<H, R, L>
    for TestConditionalCoalescenceSampler
{
    #[must_use]
    fn sample_optional_coalescence_at_location(
        &self,
        location: &Location,
        habitat: u32,
        rng: &mut impl Rng,
    ) -> Option<R> {
        unimplemented!()
    }
}
#[contract_trait]
impl<H: Habitat, R: LineageReference<H>, L: LineageSampler<H, R>>
    ConditionalCoalescenceSampler<H, R, L> for TestConditionalCoalescenceSampler
{
    #[must_use]
    fn sample_coalescence_at_location(&self, location: &Location, rng: &mut impl Rng) -> R {
        unimplemented!()
    }

    #[must_use]
    fn get_coalescence_probability_at_location(&self, location: &Location, habitat: u32) -> f64 {
        unimplemented!()
    }
}

pub struct TestUnconditionalProbability;
pub struct TestConditionalProbability;

impl<
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        L: LineageSampler<H, R>,
        C: CoalescenceSampler<H, R, L>,
    > ProbabilityStage<H, D, R, L, C> for TestUnconditionalProbability
{
}
impl<
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        L: LineageSampler<H, R>,
        C: CoalescenceSampler<H, R, L>,
    > UnconditionalProbability<H, D, R, L, C> for TestUnconditionalProbability
{
}
impl<
        H: Habitat,
        D: SeparableDispersalSampler<H>,
        R: LineageReference<H>,
        L: LineageSampler<H, R>,
        C: ConditionalCoalescenceSampler<H, R, L>,
    > ProbabilityStage<H, D, R, L, C> for TestConditionalProbability
{
}
impl<
        H: Habitat,
        D: SeparableDispersalSampler<H>,
        R: LineageReference<H>,
        L: LineageSampler<H, R>,
        C: ConditionalCoalescenceSampler<H, R, L>,
    > ConditionalProbability<H, D, R, L, C> for TestConditionalProbability
{
}

pub struct TestUnconditionalEvent;
pub struct TestConditionalEvent;

impl<
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        L: LineageSampler<H, R>,
        C: CoalescenceSampler<H, R, L>,
        P: UnconditionalProbability<H, D, R, L, C>,
    > EventStage<H, D, R, L, C, P> for TestUnconditionalEvent
{
}
impl<
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        L: LineageSampler<H, R>,
        C: CoalescenceSampler<H, R, L>,
        P: UnconditionalProbability<H, D, R, L, C>,
    > UnconditionalEvent<H, D, R, L, C, P> for TestUnconditionalEvent
{
}
impl<
        H: Habitat,
        D: SeparableDispersalSampler<H>,
        R: LineageReference<H>,
        L: LineageSampler<H, R>,
        C: ConditionalCoalescenceSampler<H, R, L>,
        P: ConditionalProbability<H, D, R, L, C>,
    > EventStage<H, D, R, L, C, P> for TestConditionalEvent
{
}
impl<
        H: Habitat,
        D: SeparableDispersalSampler<H>,
        R: LineageReference<H>,
        L: LineageSampler<H, R>,
        C: ConditionalCoalescenceSampler<H, R, L>,
        P: ConditionalProbability<H, D, R, L, C>,
    > ConditionalEvent<H, D, R, L, C, P> for TestConditionalEvent
{
}

pub fn test() {
    let simulation = super::Simulation::builder()
        .habitat(TestHabitat)
        .dispersal_sampler(TestBaseDispersalSampler)
        .lineage_reference(std::marker::PhantomData::<TestLineageReference>)
        .lineage_sampler(TestLineageSampler(std::marker::PhantomData::<TestHabitat>))
        .coalescence_sampler(TestCoalescenceSampler)
        .probability(TestUnconditionalProbability)
        .event(TestUnconditionalEvent)
        .build();
}
