use necsim_core::{
    cogs::{Backup, Habitat, TurnoverRate},
    landscape::Location,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::RustToCudaAsRust))]
#[allow(clippy::module_name_repetitions)]
pub struct UniformTurnoverRate(());

impl Default for UniformTurnoverRate {
    fn default() -> Self {
        Self(())
    }
}

#[contract_trait]
impl Backup for UniformTurnoverRate {
    unsafe fn backup_unchecked(&self) -> Self {
        Self(())
    }
}

#[contract_trait]
impl<H: Habitat> TurnoverRate<H> for UniformTurnoverRate {
    #[must_use]
    #[inline]
    fn get_turnover_rate_at_location(&self, _location: &Location, _habitat: &H) -> NonNegativeF64 {
        Self::get_uniform_turnover_rate().into()
    }
}

impl UniformTurnoverRate {
    #[must_use]
    #[inline]
    pub fn get_uniform_turnover_rate() -> PositiveF64 {
        unsafe { PositiveF64::new_unchecked(0.5_f64) }
    }
}
