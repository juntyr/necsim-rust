mod habitat;
pub use habitat::Habitat;

mod dispersal_sampler;
pub use dispersal_sampler::{DispersalSampler, SeparableDispersalSampler};

mod lineage_reference;
pub use lineage_reference::LineageReference;

mod lineage_sampler;
pub use lineage_sampler::LineageSampler;

mod coalescence_sampler;
pub use coalescence_sampler::{CoalescenceSampler, ConditionalCoalescenceSampler};

pub trait ProbabilityStage<
    H: Habitat,
    D: DispersalSampler<H>,
    R: LineageReference<H>,
    L: LineageSampler<H, R>,
    C: CoalescenceSampler<H, R, L>,
>
{
}
pub trait UnconditionalProbability<
    H: Habitat,
    D: DispersalSampler<H>,
    R: LineageReference<H>,
    L: LineageSampler<H, R>,
    C: CoalescenceSampler<H, R, L>,
>: ProbabilityStage<H, D, R, L, C>
{
}
pub trait ConditionalProbability<
    H: Habitat,
    D: SeparableDispersalSampler<H>,
    R: LineageReference<H>,
    L: LineageSampler<H, R>,
    C: ConditionalCoalescenceSampler<H, R, L>,
>: ProbabilityStage<H, D, R, L, C>
{
}

pub trait EventStage<
    H: Habitat,
    D: DispersalSampler<H>,
    R: LineageReference<H>,
    L: LineageSampler<H, R>,
    C: CoalescenceSampler<H, R, L>,
    P: ProbabilityStage<H, D, R, L, C>,
>
{
}
pub trait UnconditionalEvent<
    H: Habitat,
    D: DispersalSampler<H>,
    R: LineageReference<H>,
    L: LineageSampler<H, R>,
    C: CoalescenceSampler<H, R, L>,
    P: UnconditionalProbability<H, D, R, L, C>,
>: EventStage<H, D, R, L, C, P>
{
}
pub trait ConditionalEvent<
    H: Habitat,
    D: SeparableDispersalSampler<H>,
    R: LineageReference<H>,
    L: LineageSampler<H, R>,
    C: ConditionalCoalescenceSampler<H, R, L>,
    P: ConditionalProbability<H, D, R, L, C>,
>: EventStage<H, D, R, L, C, P>
{
}
