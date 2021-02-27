use core::marker::PhantomData;

use alloc::vec::Vec;

use necsim_core::cogs::{
    Backup, CoherentLineageStore, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
    LineageReference, RngCore, SpeciationProbability,
};

mod sampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ClassicalActiveLineageSampler<
    H: Habitat,
    G: RngCore,
    N: SpeciationProbability<H>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: CoherentLineageStore<H, R>,
    X: EmigrationExit<H, G, N, D, R, S>,
    I: ImmigrationEntry,
> {
    active_lineage_references: Vec<R>,
    last_event_time: f64,
    next_event_time: Option<f64>,
    _marker: PhantomData<(H, G, N, D, S, X, I)>,
}

impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
        I: ImmigrationEntry,
    > ClassicalActiveLineageSampler<H, G, N, D, R, S, X, I>
{
    #[must_use]
    pub fn new(lineage_store: &S) -> Self {
        Self {
            active_lineage_references: lineage_store
                .iter_local_lineage_references()
                .filter(|local_reference| lineage_store.get(local_reference.clone()).is_some())
                .collect(),
            last_event_time: 0.0_f64,
            next_event_time: None,
            _marker: PhantomData::<(H, G, N, D, S, X, I)>,
        }
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
        I: ImmigrationEntry,
    > Backup for ClassicalActiveLineageSampler<H, G, N, D, R, S, X, I>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            active_lineage_references: self.active_lineage_references.clone(),
            last_event_time: self.last_event_time,
            next_event_time: self.next_event_time,
            _marker: PhantomData::<(H, G, N, D, S, X, I)>,
        }
    }
}
