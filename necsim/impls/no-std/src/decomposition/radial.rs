use libm::atan2;

use necsim_core::{
    cogs::{Backup, Habitat, MathsCore},
    landscape::Location,
};
use necsim_core_bond::Partition;

use crate::decomposition::Decomposition;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct RadialDecomposition {
    subdomain: Partition,
}

impl RadialDecomposition {
    #[must_use]
    pub fn new(subdomain: Partition) -> Self {
        Self { subdomain }
    }
}

#[contract_trait]
impl Backup for RadialDecomposition {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            subdomain: self.subdomain,
        }
    }
}

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>> Decomposition<M, H> for RadialDecomposition {
    fn get_subdomain(&self) -> Partition {
        self.subdomain
    }

    fn map_location_to_subdomain_rank(&self, location: &Location, habitat: &H) -> u32 {
        let extent = habitat.get_extent();

        let centre_x = extent.width() / 2 + extent.x();
        let centre_y = extent.height() / 2 + extent.y();

        #[allow(clippy::cast_precision_loss)]
        let fraction = (atan2(
            (i64::from(location.y()) - i64::from(centre_y)) as f64,
            (i64::from(location.x()) - i64::from(centre_x)) as f64,
        ) * core::f64::consts::FRAC_1_PI
            * 0.5_f64)
            + 0.5_f64;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            (M::floor(f64::from(self.subdomain.size().get()) * fraction) as u32)
                .min(self.subdomain.size().get() - 1)
        }
    }
}
