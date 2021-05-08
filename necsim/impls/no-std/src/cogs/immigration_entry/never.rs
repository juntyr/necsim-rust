use necsim_core::{
    cogs::{Backup, ImmigrationEntry},
    lineage::MigratingLineage,
};
use necsim_core_bond::PositiveF64;

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[derive(Debug)]
pub struct NeverImmigrationEntry(());

impl Default for NeverImmigrationEntry {
    fn default() -> Self {
        Self(())
    }
}

#[contract_trait]
impl Backup for NeverImmigrationEntry {
    unsafe fn backup_unchecked(&self) -> Self {
        Self(())
    }
}

#[contract_trait]
impl ImmigrationEntry for NeverImmigrationEntry {
    #[must_use]
    #[inline]
    #[debug_ensures(ret.is_none(), "no lineage ever immigrates")]
    fn next_optional_immigration(
        &mut self,
        _optional_next_event_time: Option<PositiveF64>,
    ) -> Option<MigratingLineage> {
        None
    }

    #[must_use]
    #[inline]
    #[debug_ensures(ret.is_none(), "no lineage ever immigrates")]
    fn peek_next_immigration(&self) -> Option<&MigratingLineage> {
        None
    }
}
