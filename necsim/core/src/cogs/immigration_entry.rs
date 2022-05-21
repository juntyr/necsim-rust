use crate::lineage::MigratingLineage;

use super::{Backup, MathsCore};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait ImmigrationEntry<M: MathsCore>: Backup + core::fmt::Debug {
    #[must_use]
    fn next_optional_immigration(&mut self) -> Option<MigratingLineage>;

    #[must_use]
    fn peek_next_immigration(&self) -> Option<&MigratingLineage>;
}
