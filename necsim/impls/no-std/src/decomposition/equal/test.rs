use core::{convert::TryFrom, num::NonZeroU32};

use hashbrown::HashMap;

use necsim_core::cogs::{Backup, Habitat};
use necsim_core_bond::OffByOneU32;
use necsim_core_maths::IntrinsicsMathsCore;
use necsim_partitioning_core::partition::Partition;

use crate::{
    cogs::habitat::{non_spatial::NonSpatialHabitat, spatially_implicit::SpatiallyImplicitHabitat},
    decomposition::Decomposition,
};

use super::EqualDecomposition;

#[test]
fn test_equal_area_decomposition() {
    let mut indices: HashMap<u32, usize> = HashMap::with_capacity(64);

    for width in 1..=8 {
        for height in 1..=8 {
            let habitat: NonSpatialHabitat<IntrinsicsMathsCore> = NonSpatialHabitat::new(
                (
                    OffByOneU32::new(u64::from(width)).unwrap(),
                    OffByOneU32::new(u64::from(height)).unwrap(),
                ),
                NonZeroU32::new(1).unwrap(),
            );

            for partition in 1..=(width * height + 1) {
                let (successful, decomposition) = match EqualDecomposition::area(
                    &habitat,
                    Partition::try_new(0, NonZeroU32::new(partition).unwrap()).unwrap(),
                ) {
                    Ok(decomposition) => (true, decomposition),
                    Err(decomposition) => (false, decomposition),
                };

                // Test decomposition backup
                let decomposition = decomposition.backup();

                indices.clear();

                for location in habitat.iter_habitable_locations() {
                    let index = decomposition.map_location_to_subdomain_rank(&location, &habitat);

                    *indices.entry(index).or_insert(0) += 1;
                }

                let assert_message = alloc::format!(
                    "{}x{} / {} => {:?} => {}@{:?}",
                    width,
                    height,
                    partition,
                    decomposition,
                    indices.len(),
                    indices,
                );

                let num_indices = u32::try_from(indices.len()).expect(&assert_message);

                // Check that the number of generated indices is less than
                //  (unsuccessful) or equal (successful) to the partition
                if successful {
                    assert_eq!(num_indices, partition, "{}", &assert_message);
                } else {
                    assert!(num_indices > 0, "{}", assert_message);
                    assert!(num_indices < partition, "{}", assert_message);
                    assert!(
                        u64::from(num_indices) == (u64::from(width) * u64::from(height)),
                        "{}",           // GRCOV_EXCL_LINE
                        assert_message  // GRCOV_EXCL_LINE
                    );
                }

                // Check that all indices in [0, num_indices) have been assigned
                for i in 0..num_indices {
                    assert!(indices.contains_key(&i), "{}", assert_message);
                }

                let min_index_frequency = indices.iter().map(|(_, freq)| freq).min().unwrap();
                let max_index_frequency = indices.iter().map(|(_, freq)| freq).max().unwrap();

                // Check that the indices are distributed equally
                assert!(
                    (max_index_frequency - min_index_frequency) <= 1,
                    "{}",           // GRCOV_EXCL_LINE
                    assert_message  // GRCOV_EXCL_LINE
                );
            }
        }
    }
}

#[test]
fn test_equal_weight_decomposition() {
    let mut indices: HashMap<u32, usize> = HashMap::with_capacity(64);

    for local in 1..=8 {
        for meta in 1..=8 {
            let habitat: SpatiallyImplicitHabitat<IntrinsicsMathsCore> =
                SpatiallyImplicitHabitat::new(
                    (OffByOneU32::new(8).unwrap(), OffByOneU32::new(1).unwrap()),
                    NonZeroU32::new(local).unwrap(),
                    (OffByOneU32::new(8).unwrap(), OffByOneU32::new(1).unwrap()),
                    NonZeroU32::new(meta).unwrap(),
                );

            for partition in 1..=(local * 8 + meta * 8 + 1) {
                let (successful, decomposition) = match EqualDecomposition::weight(
                    &habitat,
                    Partition::try_new(0, NonZeroU32::new(partition).unwrap()).unwrap(),
                ) {
                    Ok(decomposition) => (true, decomposition),
                    Err(decomposition) => (false, decomposition),
                };

                // Test decomposition backup
                let decomposition = decomposition.backup();

                indices.clear();

                for location in habitat.iter_habitable_locations() {
                    let h = habitat.get_habitat_at_location(&location);

                    if h > 0 {
                        let index =
                            decomposition.map_location_to_subdomain_rank(&location, &habitat);

                        *indices.entry(index).or_insert(0) += h as usize;
                    }
                }

                let assert_message = alloc::format!(
                    "{}x{}->{}x{} / {} => {:?} => {}@{:?}",
                    8,
                    local,
                    8,
                    meta,
                    partition,
                    decomposition,
                    indices.len(),
                    indices,
                );

                let num_indices = u32::try_from(indices.len()).expect(&assert_message);

                // Check that the number of generated indices is less than
                //  (unsuccessful) or equal (successful) to the partition
                if successful {
                    assert_eq!(num_indices, partition, "{}", &assert_message);
                } else {
                    assert!(num_indices > 0, "{}", assert_message);
                    assert!(num_indices < partition, "{}", assert_message);
                }

                // Check that all indices in [0, num_indices) have been assigned
                for i in 0..num_indices {
                    assert!(indices.contains_key(&i), "{}", assert_message);
                }

                let min_index_frequency = indices.iter().map(|(_, freq)| freq).min().unwrap();
                let max_index_frequency = indices.iter().map(|(_, freq)| freq).max().unwrap();

                // Check that the indices are distributed equally
                assert!(
                    (max_index_frequency - min_index_frequency) <= (local.max(meta) * 2) as usize,
                    "{}",           // GRCOV_EXCL_LINE
                    assert_message  // GRCOV_EXCL_LINE
                );
            }
        }
    }
}

#[test]
fn equal_area_stores_subdomain() {
    let habitat: NonSpatialHabitat<IntrinsicsMathsCore> = NonSpatialHabitat::new(
        (
            OffByOneU32::new(100).unwrap(),
            OffByOneU32::new(100).unwrap(),
        ),
        NonZeroU32::new(100).unwrap(),
    );

    let equal_area_decomposition = EqualDecomposition::area(
        &habitat,
        Partition::try_new(42, NonZeroU32::new(100).unwrap()).unwrap(),
    )
    .unwrap()
    .backup();

    assert_eq!(equal_area_decomposition.get_subdomain().rank(), 42);
    assert_eq!(equal_area_decomposition.get_subdomain().size().get(), 100);
}

#[test]
fn equal_weight_stores_subdomain() {
    let habitat: NonSpatialHabitat<IntrinsicsMathsCore> = NonSpatialHabitat::new(
        (
            OffByOneU32::new(100).unwrap(),
            OffByOneU32::new(100).unwrap(),
        ),
        NonZeroU32::new(100).unwrap(),
    );

    let equal_area_decomposition = EqualDecomposition::area(
        &habitat,
        Partition::try_new(24, NonZeroU32::new(1000).unwrap()).unwrap(),
    )
    .unwrap()
    .backup();

    assert_eq!(equal_area_decomposition.get_subdomain().rank(), 24);
    assert_eq!(equal_area_decomposition.get_subdomain().size().get(), 1000);
}
