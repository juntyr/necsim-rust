use core::num::NonZeroU32;

use libm::atan2;

use necsim_core::{
    cogs::{Backup, Habitat, F64Core},
    landscape::Location,
};

use crate::decomposition::Decomposition;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct RadialDecomposition {
    rank: u32,
    partitions: NonZeroU32,
}

impl RadialDecomposition {
    #[must_use]
    pub fn new(rank: u32, partitions: NonZeroU32) -> Self {
        Self { rank, partitions }
    }
}

#[contract_trait]
impl Backup for RadialDecomposition {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            rank: self.rank,
            partitions: self.partitions,
        }
    }
}

#[contract_trait]
impl<F: F64Core, H: Habitat<F>> Decomposition<F, H> for RadialDecomposition {
    fn get_subdomain_rank(&self) -> u32 {
        self.rank
    }

    fn get_number_of_subdomains(&self) -> NonZeroU32 {
        self.partitions
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
            (F::floor(f64::from(self.partitions.get()) * fraction) as u32)
                .min(self.partitions.get() - 1)
        }
    }
}
