use core::marker::PhantomData;

use crate::cogs::{Habitat, LineageReference, LineageStore, MathsCore, RngCore};

#[repr(C)]
pub struct PartialSimulation<
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    R: LineageReference<M, H>,
    S: LineageStore<M, H, R>,
> {
    pub maths: PhantomData<M>,
    pub habitat: H,
    pub lineage_reference: PhantomData<R>,
    pub lineage_store: S,
    // priv
    _rng: PhantomData<G>,
}
