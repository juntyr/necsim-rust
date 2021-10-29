use core::marker::PhantomData;

use alloc::vec::Vec;

use necsim_core::cogs::{
    Backup, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry, LineageReference,
    LocallyCoherentLineageStore, MathsCore, RngCore, SpeciationProbability,
};
use necsim_core_bond::NonNegativeF64;

mod sampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ClassicalActiveLineageSampler<
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    R: LineageReference<M, H>,
    S: LocallyCoherentLineageStore<M, H, R>,
    X: EmigrationExit<M, H, G, R, S>,
    D: DispersalSampler<M, H, G>,
    N: SpeciationProbability<M, H>,
    I: ImmigrationEntry<M>,
> {
    active_lineage_references: Vec<R>,
    last_event_time: NonNegativeF64,
    #[allow(clippy::type_complexity)]
    _marker: PhantomData<(M, H, G, S, X, D, N, I)>,
}

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: LocallyCoherentLineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        N: SpeciationProbability<M, H>,
        I: ImmigrationEntry<M>,
    > ClassicalActiveLineageSampler<M, H, G, R, S, X, D, N, I>
{
    #[must_use]
    pub fn new(lineage_store: &S) -> Self {
        let mut last_event_time = NonNegativeF64::zero();

        Self {
            active_lineage_references: lineage_store
                .iter_local_lineage_references()
                .inspect(|local_reference| {
                    last_event_time =
                        last_event_time.max(lineage_store[local_reference.clone()].last_event_time);
                })
                .collect(),
            last_event_time,
            _marker: PhantomData::<(M, H, G, S, X, D, N, I)>,
        }
    }
}

#[contract_trait]
impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: LocallyCoherentLineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        N: SpeciationProbability<M, H>,
        I: ImmigrationEntry<M>,
    > Backup for ClassicalActiveLineageSampler<M, H, G, R, S, X, D, N, I>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            active_lineage_references: self.active_lineage_references.clone(),
            last_event_time: self.last_event_time,
            _marker: PhantomData::<(M, H, G, S, X, D, N, I)>,
        }
    }
}
