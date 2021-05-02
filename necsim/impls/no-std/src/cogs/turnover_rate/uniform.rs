use necsim_core::{
    cogs::{Backup, Habitat, TurnoverRate},
    landscape::Location,
};

#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
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
    fn get_turnover_rate_at_location(&self, _location: &Location, _habitat: &H) -> f64 {
        Self::get_uniform_turnover_rate()
    }
}

impl UniformTurnoverRate {
    #[must_use]
    #[inline]
    pub fn get_uniform_turnover_rate() -> f64 {
        0.5_f64
    }
}
