use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, F64Core, Habitat,
        LineageReference, LineageStore, RngCore, SpeciationProbability, TurnoverRate,
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
    F: F64Core,
    H: Habitat<F>,
    G: RngCore<F>,
    R: LineageReference<F, H>,
    S: LineageStore<F, H, R>,
    X: EmigrationExit<F, H, G, R, S>,
    D: DispersalSampler<F, H, G>,
    C: CoalescenceSampler<F, H, R, S>,
    T: TurnoverRate<F, H>,
    N: SpeciationProbability<F, H>,
>: EventSampler<F, H, G, R, S, X, D, C, T, N>
{
    /// Pre: all lineages must be active in the lineage store
    #[must_use]
    fn get_event_rate_at_location(
        &self,
        location: &Location,
        simulation: &GillespiePartialSimulation<F, H, G, R, S, D, C, T, N>,
    ) -> NonNegativeF64;
}

#[repr(C)]
#[allow(clippy::module_name_repetitions)]
pub struct GillespiePartialSimulation<
    F: F64Core,
    H: Habitat<F>,
    G: RngCore<F>,
    R: LineageReference<F, H>,
    S: LineageStore<F, H, R>,
    D: DispersalSampler<F, H, G>,
    C: CoalescenceSampler<F, H, R, S>,
    T: TurnoverRate<F, H>,
    N: SpeciationProbability<F, H>,
> {
    pub f64_core: PhantomData<F>,
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
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: LineageStore<F, H, R>,
        D: DispersalSampler<F, H, G>,
        C: CoalescenceSampler<F, H, R, S>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
    > GillespiePartialSimulation<F, H, G, R, S, D, C, T, N>
{
    #[inline]
    pub fn without_emigration_exit<X: EmigrationExit<F, H, G, R, S>, Q, W: FnOnce(&Self) -> Q>(
        super_partial: &necsim_core::simulation::partial::event_sampler::PartialSimulation<
            F,
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
        func: W,
    ) -> Q {
        // Cast &mut self to a &mut PartialSimulation without the emigration exit
        // This is only safe as PartialSimulation's type and layout is a prefix
        //  subsequence of Self's type and layout
        let partial_simulation = unsafe {
            &*(super_partial
                as *const necsim_core::simulation::partial::event_sampler::PartialSimulation<
                    F,
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
