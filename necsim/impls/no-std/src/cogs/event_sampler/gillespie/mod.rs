use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, Habitat,
        LineageReference, LineageStore, MathsCore, RngCore, SpeciationProbability, TurnoverRate,
    },
    landscape::Location,
};
use necsim_core_bond::NonNegativeF64;

pub mod conditional;
pub mod unconditional;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
#[allow(clippy::module_name_repetitions)]
pub trait GillespieEventSampler<
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    R: LineageReference<M, H>,
    S: LineageStore<M, H, R>,
    X: EmigrationExit<M, H, G, R, S>,
    D: DispersalSampler<M, H, G>,
    C: CoalescenceSampler<M, H, R, S>,
    T: TurnoverRate<M, H>,
    N: SpeciationProbability<M, H>,
>: EventSampler<M, H, G, R, S, X, D, C, T, N>
{
    /// Pre: all lineages must be active in the lineage store
    #[must_use]
    fn get_event_rate_at_location(
        &self,
        location: &Location,
        simulation: &GillespiePartialSimulation<M, H, G, R, S, D, C, T, N>,
    ) -> NonNegativeF64;
}

#[repr(C)]
#[allow(clippy::module_name_repetitions)]
pub struct GillespiePartialSimulation<
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    R: LineageReference<M, H>,
    S: LineageStore<M, H, R>,
    D: DispersalSampler<M, H, G>,
    C: CoalescenceSampler<M, H, R, S>,
    T: TurnoverRate<M, H>,
    N: SpeciationProbability<M, H>,
> {
    pub maths: PhantomData<M>,
    pub habitat: H,
    pub lineage_reference: PhantomData<R>,
    pub lineage_store: S,
    pub dispersal_sampler: D,
    pub coalescence_sampler: C,
    pub turnover_rate: T,
    pub speciation_probability: N,
    pub _rng: PhantomData<G>,
}

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: LineageStore<M, H, R>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
    > GillespiePartialSimulation<M, H, G, R, S, D, C, T, N>
{
    #[inline]
    pub fn without_emigration_exit<X: EmigrationExit<M, H, G, R, S>, Q, F: FnOnce(&Self) -> Q>(
        super_partial: &necsim_core::simulation::partial::event_sampler::PartialSimulation<
            M,
            H,
            G,
            R,
            S,
            X,
            D,
            C,
            T,
            N,
        >,
        func: F,
    ) -> Q {
        // Cast &mut self to a &mut PartialSimulation without the emigration exit
        // This is only safe as PartialSimulation's type and layout is a prefix
        //  subsequence of Self's type and layout
        let partial_simulation = unsafe {
            &*(super_partial
                as *const necsim_core::simulation::partial::event_sampler::PartialSimulation<
                    M,
                    H,
                    G,
                    R,
                    S,
                    X,
                    D,
                    C,
                    T,
                    N,
                >)
                .cast::<Self>()
        };

        func(partial_simulation)
    }
}
