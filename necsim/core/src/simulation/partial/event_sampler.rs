use core::marker::PhantomData;

use crate::cogs::{
    CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, LineageReference, LineageStore,
    RngCore, SpeciationProbability,
};

#[repr(C)]
pub struct PartialSimulation<
    H: Habitat,
    G: RngCore,
    N: SpeciationProbability<H>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    X: EmigrationExit<H, G, N, D, R, S>,
    C: CoalescenceSampler<H, G, R, S>,
> {
    pub habitat: H,
    pub speciation_probability: N,
    pub dispersal_sampler: D,
    pub lineage_reference: PhantomData<R>,
    pub lineage_store: S,
    pub emigration_exit: X,
    pub coalescence_sampler: C,
    pub rng: PhantomData<G>,
}

impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
        C: CoalescenceSampler<H, G, R, S>,
    > PartialSimulation<H, G, N, D, R, S, X, C>
{
    #[inline]
    pub fn with_mut_split_emigration_exit<
        Q,
        F: FnOnce(&mut X, &mut super::migration::PartialSimulation<H, G, N, D, R, S>) -> Q,
    >(
        &mut self,
        func: F,
    ) -> Q {
        // Cast &mut self to a &mut PartialSimulation without the emigration exit
        // This is only safe as both types have the same fields and layout except for
        // emigration exit (and coalescence sampler) in Self at the end
        #[allow(clippy::cast_ref_to_mut)]
        let partial_simulation = unsafe {
            &mut *(self as *const Self
                as *mut super::migration::PartialSimulation<H, G, N, D, R, S>)
        };

        func(&mut self.emigration_exit, partial_simulation)
    }
}
