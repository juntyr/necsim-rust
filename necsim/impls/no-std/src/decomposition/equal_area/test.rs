use alloc::vec::Vec;
use core::num::NonZeroU32;

use necsim_core::cogs::Habitat;

use crate::{cogs::habitat::non_spatial::NonSpatialHabitat, decomposition::Decomposition};

use super::EqualAreaDecomposition;

#[test]
fn test_ok() {
    let mut results: Vec<((u32, u32, u32), (u32, u32, u32))> = Vec::new();

    for width in 1..=7 {
        for height in 1..=7 {
            let habitat = NonSpatialHabitat::new((width, height), 1);

            for partition in 1..=(width * height + 1) {
                let decomposition = match EqualAreaDecomposition::new(
                    &habitat,
                    0,
                    NonZeroU32::new(partition).unwrap(),
                ) {
                    Ok(decomposition) | Err(decomposition) => decomposition,
                };

                for location in habitat.get_extent().iter() {
                    let index = decomposition.map_location_to_subdomain_rank(&location, &habitat);

                    results.push((
                        (width, height, partition),
                        (location.x(), location.y(), index),
                    ));
                }
            }
        }
    }

    panic!("{:?}", results);
}
