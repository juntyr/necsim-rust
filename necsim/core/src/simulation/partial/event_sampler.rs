use core::marker::PhantomData;

use crate::cogs::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, LineageReference, LineageStore,
    RngCore, SpeciationProbability, TurnoverRate,
};

#[repr(C)]
pub struct PartialSimulation<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    X: EmigrationExit<H, G, R, S>,
    D: DispersalSampler<H, G>,
    C: CoalescenceSampler<H, R, S>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
> {
    pub habitat: H,
    pub lineage_reference: PhantomData<R>,
    pub lineage_store: S,
    pub dispersal_sampler: D,
    pub coalescence_sampler: C,
    pub turnover_rate: T,
    pub speciation_probability: N,
    pub emigration_exit: X,
    // priv
    _rng: PhantomData<G>,
}

impl<
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
    > PartialSimulation<H, G, R, S, X, D, C, T, N>
{
    #[inline]
    pub fn with_mut_split_emigration_exit<
        Q,
        F: FnOnce(&mut X, &mut super::emigration_exit::PartialSimulation<H, G, R, S>) -> Q,
    >(
        &mut self,
        func: F,
    ) -> Q {
        // Cast &mut self to a &mut PartialSimulation without the emigration exit
        // This is only safe as PartialSimulation's type and layout is a prefix
        //  subsequence of Self's type and layout
        let partial_simulation = unsafe {
            &mut *(self as *mut Self)
                .cast::<super::emigration_exit::PartialSimulation<H, G, R, S>>()
        };

        func(&mut self.emigration_exit, partial_simulation)
    }
}
