use core::marker::PhantomData;

use crate::cogs::{Habitat, LineageReference, LineageStore, RngCore};

#[repr(C)]
pub struct PartialSimulation<H: Habitat, G: RngCore, R: LineageReference<H>, S: LineageStore<H, R>>
{
    pub habitat: H,
    pub lineage_reference: PhantomData<R>,
    pub lineage_store: S,
    // priv
    _rng: PhantomData<G>,
}
