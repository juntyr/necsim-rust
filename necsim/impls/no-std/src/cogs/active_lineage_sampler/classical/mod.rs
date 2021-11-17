use core::marker::PhantomData;

use alloc::vec::Vec;

use necsim_core::cogs::{
    Backup, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry, LineageReference,
    LocallyCoherentLineageStore, MathsCore, RngCore, SpeciationProbability,
};
use necsim_core_bond::NonNegativeF64;

use crate::cogs::origin_sampler::OriginSampler;

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
    pub fn new_with_store<'h, O: OriginSampler<'h, M, Habitat = H>>(
        mut origin_sampler: O,
    ) -> (S, Self)
    where
        H: 'h,
    {
        #[allow(clippy::cast_possible_truncation)]
        let capacity = origin_sampler.full_upper_bound_size_hint() as usize;

        let mut lineage_store = S::with_capacity(origin_sampler.habitat(), capacity);

        let mut active_lineage_references = Vec::with_capacity(capacity);
        let mut last_event_time = NonNegativeF64::zero();

        while let Some(lineage) = origin_sampler.next() {
            last_event_time = last_event_time.max(lineage.last_event_time);

            active_lineage_references.push(
                lineage_store.insert_lineage_locally_coherent(lineage, origin_sampler.habitat()),
            );
        }

        (
            lineage_store,
            Self {
                active_lineage_references,
                last_event_time,
                _marker: PhantomData::<(M, H, G, S, X, D, N, I)>,
            },
        )
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
