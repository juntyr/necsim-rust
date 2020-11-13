use core::convert::TryFrom;

use crate::{
    cogs::{CoherentLineageStore, Habitat, LineageReference},
    landscape::Location,
};

#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn explicit_lineage_store_lineage_at_location_contract<
    H: Habitat,
    R: LineageReference<H>,
    L: CoherentLineageStore<H, R>,
>(
    store: &L,
    reference: R,
) -> bool {
    let input_reference = reference.clone();

    let lineage = match store.get(reference) {
        Some(lineage) => lineage,
        None => return false,
    };

    let indexed_location = match lineage.indexed_location() {
        Some(indexed_location) => indexed_location,
        None => return false,
    };

    let lineages_at_location = &store.get_active_lineages_at_location(indexed_location.location());

    match lineages_at_location.get(indexed_location.index() as usize) {
        Some(reference_at_location) => reference_at_location == &input_reference,
        None => false,
    }
}

#[must_use]
pub(super) fn explicit_lineage_store_invariant_contract<
    H: Habitat,
    R: LineageReference<H>,
    L: CoherentLineageStore<H, R>,
>(
    store: &L,
    location: &Location,
) -> bool {
    let lineages_at_location = &store.get_active_lineages_at_location(location);

    lineages_at_location
        .iter()
        .enumerate()
        .all(|(index, reference)| {
            match (
                u32::try_from(index),
                store[reference.clone()].indexed_location(),
            ) {
                (Ok(index), Some(indexed_location)) => {
                    indexed_location.location() == location && indexed_location.index() == index
                },
                _ => false,
            }
        })
}
