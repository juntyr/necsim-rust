#[allow(clippy::module_name_repetitions)]
pub struct SimulationBuilder;

impl SimulationBuilder {
    #[must_use]
    pub fn with_parameters(
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
    ) -> SimulationBuilderWithParameters {
        SimulationBuilderWithParameters {
            speciation_probability_per_generation,
            sample_percentage,
        }
    }
}

pub struct SimulationBuilderWithParameters {
    speciation_probability_per_generation: f64,
    sample_percentage: f64,
}

impl SimulationBuilderWithParameters {
    #[must_use]
    pub fn with_habitat<H: Habitat>(self, habitat: H) -> SimulationBuilderWithParametersHabitat<H> {
        SimulationBuilderWithParametersHabitat {
            speciation_probability_per_generation: self.speciation_probability_per_generation,
            sample_percentage: self.sample_percentage,
            habitat,
        }
    }
}

pub trait Habitat {}

pub struct TestHabitat;
impl Habitat for TestHabitat {}

pub struct SimulationBuilderWithParametersHabitat<H: Habitat> {
    speciation_probability_per_generation: f64,
    sample_percentage: f64,
    habitat: H,
}

impl<H: Habitat> SimulationBuilderWithParametersHabitat<H> {
    #[must_use]
    pub fn with_dispersal<D: Dispersal>(
        self,
        dispersal: D,
    ) -> SimulationBuilderWithParametersHabitatDispersal<H, D> {
        SimulationBuilderWithParametersHabitatDispersal {
            speciation_probability_per_generation: self.speciation_probability_per_generation,
            sample_percentage: self.sample_percentage,
            habitat: self.habitat,
            dispersal,
        }
    }
}

pub trait Dispersal {}

pub struct TestDispersal;
impl Dispersal for TestDispersal {}

pub trait SeparableDispersal: Dispersal {}

pub struct TestSeparableDispersal;
impl Dispersal for TestSeparableDispersal {}
impl SeparableDispersal for TestSeparableDispersal {}

pub struct SimulationBuilderWithParametersHabitatDispersal<H: Habitat, D: Dispersal> {
    speciation_probability_per_generation: f64,
    sample_percentage: f64,
    habitat: H,
    dispersal: D,
}

impl<H: Habitat, D: Dispersal> SimulationBuilderWithParametersHabitatDispersal<H, D> {
    #[must_use]
    pub fn with_lineage_sampler_and_reference<R: LineageReference, L: LineageSampler<R>>(
        self,
        lineage_sampler: L,
    ) -> SimulationBuilderWithParametersHabitatDispersalLineage<H, D, R, L> {
        SimulationBuilderWithParametersHabitatDispersalLineage {
            speciation_probability_per_generation: self.speciation_probability_per_generation,
            sample_percentage: self.sample_percentage,
            habitat: self.habitat,
            dispersal: self.dispersal,
            lineage_reference: std::marker::PhantomData,
            lineage_sampler,
        }
    }
}

pub trait LineageReference {}
pub trait LineageSampler<R: LineageReference> {}

pub struct TestLineageReference;
impl LineageReference for TestLineageReference {}
pub struct TestLineageSampler;
impl LineageSampler<TestLineageReference> for TestLineageSampler {}

pub struct SimulationBuilderWithParametersHabitatDispersalLineage<
    H: Habitat,
    D: Dispersal,
    R: LineageReference,
    L: LineageSampler<R>,
> {
    speciation_probability_per_generation: f64,
    sample_percentage: f64,
    habitat: H,
    dispersal: D,
    lineage_reference: std::marker::PhantomData<R>,
    lineage_sampler: L,
}

impl<H: Habitat, D: Dispersal, R: LineageReference, L: LineageSampler<R>>
    SimulationBuilderWithParametersHabitatDispersalLineage<H, D, R, L>
{
    #[must_use]
    pub fn with_coalescence_sampler<C: CoalescenceSampler<R>>(
        self,
        coalescence_sampler: C,
    ) -> SimulationBuilderWithParametersHabitatDispersalLineageCoalescence<H, D, R, L, C> {
        SimulationBuilderWithParametersHabitatDispersalLineageCoalescence {
            speciation_probability_per_generation: self.speciation_probability_per_generation,
            sample_percentage: self.sample_percentage,
            habitat: self.habitat,
            dispersal: self.dispersal,
            lineage_reference: self.lineage_reference,
            lineage_sampler: self.lineage_sampler,
            coalescence_sampler,
        }
    }
}

pub trait CoalescenceSampler<R: LineageReference> {}

pub struct TestCoalescenceSampler;
impl CoalescenceSampler<TestLineageReference> for TestCoalescenceSampler {}

pub trait ConditionalCoalescenceSampler<R: LineageReference>: CoalescenceSampler<R> {}

pub struct TestConditionalCoalescenceSampler;
impl CoalescenceSampler<TestLineageReference> for TestConditionalCoalescenceSampler {}
impl ConditionalCoalescenceSampler<TestLineageReference> for TestConditionalCoalescenceSampler {}

pub struct SimulationBuilderWithParametersHabitatDispersalLineageCoalescence<
    H: Habitat,
    D: Dispersal,
    R: LineageReference,
    L: LineageSampler<R>,
    C: CoalescenceSampler<R>,
> {
    speciation_probability_per_generation: f64,
    sample_percentage: f64,
    habitat: H,
    dispersal: D,
    lineage_reference: std::marker::PhantomData<R>,
    lineage_sampler: L,
    coalescence_sampler: C,
}

pub fn test() {
    let builder = SimulationBuilder::with_parameters(0.1, 1.0)
        .with_habitat(TestHabitat)
        .with_dispersal(TestSeparableDispersal)
        .with_lineage_sampler_and_reference(TestLineageSampler)
        .with_coalescence_sampler(TestConditionalCoalescenceSampler);
}
