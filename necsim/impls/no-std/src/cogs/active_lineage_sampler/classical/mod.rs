use core::marker::PhantomData;

use alloc::vec::Vec;

use necsim_core::cogs::{
    Backup, DispersalSampler, EmigrationExit, F64Core, Habitat, ImmigrationEntry, LineageReference,
    LocallyCoherentLineageStore, RngCore, SpeciationProbability,
};
use necsim_core_bond::NonNegativeF64;

mod sampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ClassicalActiveLineageSampler<
    F: F64Core,
    H: Habitat<F>,
    G: RngCore<F>,
    R: LineageReference<F, H>,
    S: LocallyCoherentLineageStore<F, H, R>,
    X: EmigrationExit<F, H, G, R, S>,
    D: DispersalSampler<F, H, G>,
    N: SpeciationProbability<F, H>,
    I: ImmigrationEntry<F>,
> {
    active_lineage_references: Vec<R>,
    last_event_time: NonNegativeF64,
    #[allow(clippy::type_complexity)]
    _marker: PhantomData<(F, H, G, S, X, D, N, I)>,
}

impl<
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: LocallyCoherentLineageStore<F, H, R>,
        X: EmigrationExit<F, H, G, R, S>,
        D: DispersalSampler<F, H, G>,
        N: SpeciationProbability<F, H>,
        I: ImmigrationEntry<F>,
    > ClassicalActiveLineageSampler<F, H, G, R, S, X, D, N, I>
{
    #[must_use]
    pub fn new(lineage_store: &S) -> Self {
        Self {
            active_lineage_references: lineage_store
                .iter_local_lineage_references()
                .filter(|local_reference| {
                    lineage_store
                        .get_lineage_for_local_reference(local_reference.clone())
                        .is_some()
                })
                .collect(),
            last_event_time: NonNegativeF64::zero(),
            _marker: PhantomData::<(F, H, G, S, X, D, N, I)>,
        }
    }
}

#[contract_trait]
impl<
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: LocallyCoherentLineageStore<F, H, R>,
        X: EmigrationExit<F, H, G, R, S>,
        D: DispersalSampler<F, H, G>,
        N: SpeciationProbability<F, H>,
        I: ImmigrationEntry<F>,
    > Backup for ClassicalActiveLineageSampler<F, H, G, R, S, X, D, N, I>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            active_lineage_references: self.active_lineage_references.clone(),
            last_event_time: self.last_event_time,
            _marker: PhantomData::<(F, H, G, S, X, D, N, I)>,
        }
    }
}
