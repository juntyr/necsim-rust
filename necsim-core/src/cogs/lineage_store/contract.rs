use crate::cogs::{CoherentLineageStore, Habitat, LineageReference};
use crate::landscape::Location;

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

    let location = match lineage.location() {
        Some(location) => location,
        None => return false,
    };

    let lineages_at_location = &store.get_active_lineages_at_location(location);

    let index_at_location = match lineage.index_at_location() {
        Some(index_at_location) => index_at_location,
        None => return false,
    };

    match lineages_at_location.get(index_at_location) {
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
        .all(|(i, reference)| {
            store[reference.clone()].location() == Some(location)
                && store[reference.clone()].index_at_location() == Some(i)
        })
}
