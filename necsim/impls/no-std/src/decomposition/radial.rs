use libm::atan2;

use necsim_core::{
    cogs::{Backup, Habitat, MathsCore},
    landscape::Location,
};
use necsim_partitioning_core::partition::Partition;

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
        const BELOW_ONE: f64 = f64::from_bits(0x3FEF_FFFF_FFFF_FFFF_u64);

        let extent = habitat.get_extent();

        let neutral_x = location.x().wrapping_sub(extent.x());
        let neutral_y = location.y().wrapping_sub(extent.y());

        #[allow(clippy::cast_precision_loss)]
        let fraction = (atan2(
            (i64::from(neutral_y) - i64::from(extent.height()) / 2) as f64,
            (i64::from(neutral_x) - i64::from(extent.width()) / 2) as f64,
        ) * core::f64::consts::FRAC_1_PI
            * 0.5_f64)
            + 0.5_f64;

        let fraction = fraction.clamp(0.0_f64, BELOW_ONE);

        // Safety: [0, 1) * subdomain.size in [0, 2^32) is losslessly
        //         represented in both f64 and u32
        unsafe {
            M::floor(fraction * f64::from(self.subdomain.size().get())).to_int_unchecked::<u32>()
        }
    }
}
