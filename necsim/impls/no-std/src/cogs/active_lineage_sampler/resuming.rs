use necsim_core::lineage::Lineage;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum ExceptionalLineage {
    Coalescence(Lineage),
    OutOfDeme(Lineage),
    OutOfHabitat(Lineage),
}
