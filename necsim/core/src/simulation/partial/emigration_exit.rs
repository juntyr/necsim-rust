use core::marker::PhantomData;

use crate::cogs::{Habitat, LineageStore, MathsCore, RngCore};

#[repr(C)]
pub struct PartialSimulation<M: MathsCore, H: Habitat<M>, G: RngCore<M>, S: LineageStore<M, H>> {
    pub maths: PhantomData<M>,
    pub habitat: H,
    pub lineage_store: S,
    // priv
    _rng: PhantomData<G>,
}
