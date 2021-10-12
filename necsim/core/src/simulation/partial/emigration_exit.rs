use core::marker::PhantomData;

use crate::cogs::{Habitat, LineageReference, LineageStore, RngCore, F64Core};

#[repr(C)]
pub struct PartialSimulation<F: F64Core, H: Habitat<F>, G: RngCore<F>, R: LineageReference<F, H>, S: LineageStore<F, H, R>>
{
    pub f64_core: PhantomData<F>,
    pub habitat: H,
    pub lineage_reference: PhantomData<R>,
    pub lineage_store: S,
    // priv
    _rng: PhantomData<G>,
}
