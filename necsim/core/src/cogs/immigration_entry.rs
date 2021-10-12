use crate::lineage::MigratingLineage;

use super::F64Core;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait ImmigrationEntry<F: F64Core>: crate::cogs::Backup + core::fmt::Debug {
    #[must_use]
    fn next_optional_immigration(&mut self) -> Option<MigratingLineage>;

    #[must_use]
    fn peek_next_immigration(&self) -> Option<&MigratingLineage>;
}
