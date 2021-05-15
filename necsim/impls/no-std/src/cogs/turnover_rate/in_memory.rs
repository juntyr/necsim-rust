use alloc::boxed::Box;

use necsim_core::{
    cogs::{Backup, Habitat, MathsCore, TurnoverRate},
    landscape::Location,
};
use necsim_core_bond::NonNegativeF64;

use crate::{array2d::Array2D, cogs::habitat::in_memory::InMemoryHabitat};

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[derive(Debug)]
pub struct InMemoryTurnoverRate {
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    turnover_rate: Box<[NonNegativeF64]>,
}

#[contract_trait]
impl Backup for InMemoryTurnoverRate {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            turnover_rate: self.turnover_rate.clone(),
        }
    }
}

#[contract_trait]
impl<M: MathsCore> TurnoverRate<M, InMemoryHabitat<M>> for InMemoryTurnoverRate {
    #[must_use]
    #[inline]
    fn get_turnover_rate_at_location(
        &self,
        location: &Location,
        habitat: &InMemoryHabitat<M>,
    ) -> NonNegativeF64 {
        let extent = habitat.get_extent();

        self.turnover_rate
            .get(
                ((location.y() - extent.y()) as usize) * (extent.width() as usize)
                    + ((location.x() - extent.x()) as usize),
            )
            .copied()
            .unwrap_or_else(NonNegativeF64::zero)
    }
}

impl InMemoryTurnoverRate {
    #[must_use]
    pub fn new(turnover_rate: Array2D<NonNegativeF64>) -> Self {
        // TODO: Still needs verification against habitat

        Self {
            turnover_rate: turnover_rate.into_row_major().into_boxed_slice(),
        }
    }
}
