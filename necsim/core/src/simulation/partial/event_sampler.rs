use core::marker::PhantomData;

use crate::cogs::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, F64Core, Habitat, LineageReference,
    LineageStore, RngCore, SpeciationProbability, TurnoverRate,
};

#[repr(C)]
pub struct PartialSimulation<
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
> {
    pub f64_core: PhantomData<F>,
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
    > PartialSimulation<F, H, G, R, S, X, D, C, T, N>
{
    #[inline]
    pub fn with_mut_split_emigration_exit<
        Q,
        W: FnOnce(&mut X, &mut super::emigration_exit::PartialSimulation<F, H, G, R, S>) -> Q,
    >(
        &mut self,
        func: W,
    ) -> Q {
        // Cast &mut self to a &mut PartialSimulation without the emigration exit
        // This is only safe as PartialSimulation's type and layout is a prefix
        //  subsequence of Self's type and layout
        let partial_simulation = unsafe {
            &mut *(self as *mut Self)
                .cast::<super::emigration_exit::PartialSimulation<F, H, G, R, S>>()
        };

        func(&mut self.emigration_exit, partial_simulation)
    }
}
