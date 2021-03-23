use crate::{cogs::Habitat, landscape::Location};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait TurnoverRate<H: Habitat>: crate::cogs::Backup + core::fmt::Debug {
    #[must_use]
    #[debug_requires(habitat.contains(location), "location is inside habitat")]
    #[debug_ensures(ret > 0.0_f64, "returns a positive rate")]
    fn get_turnover_rate_at_location(&self, location: &Location, habitat: &H) -> f64;
}
