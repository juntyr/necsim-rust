use necsim_core::{
    cogs::{Backup, ImmigrationEntry},
    lineage::MigratingLineage,
};

use alloc::collections::BinaryHeap;
use core::cmp::Reverse;
use necsim_core_bond::PositiveF64;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct BufferedImmigrationEntry {
    immigrants: BinaryHeap<Reverse<MigratingLineage>>,
}

#[contract_trait]
impl Backup for BufferedImmigrationEntry {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            immigrants: self
                .immigrants
                .iter()
                .map(|migrating_lineage| Reverse(migrating_lineage.0.backup_unchecked()))
                .collect(),
        }
    }
}

#[contract_trait]
impl ImmigrationEntry for BufferedImmigrationEntry {
    #[must_use]
    fn next_optional_immigration(
        &mut self,
        optional_next_event_time: Option<PositiveF64>,
    ) -> Option<MigratingLineage> {
        let next_immigration = self.immigrants.peek()?;

        if let Some(next_event_time) = optional_next_event_time {
            if next_immigration.0.event_time > next_event_time {
                return None;
            }
        }

        self.immigrants.pop().map(|rev| rev.0)
    }

    #[must_use]
    fn peek_next_immigration(&self) -> Option<&MigratingLineage> {
        self.immigrants.peek().map(|immigrant| &immigrant.0)
    }
}

impl Default for BufferedImmigrationEntry {
    fn default() -> Self {
        Self {
            immigrants: BinaryHeap::new(),
        }
    }
}

impl BufferedImmigrationEntry {
    pub fn reset(&mut self) {
        self.immigrants.clear();
    }

    pub fn push(&mut self, immigrant: MigratingLineage) {
        self.immigrants.push(Reverse(immigrant));
    }
}
