use alloc::vec::Vec;

use necsim_core::lineage::Lineage;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum ExceptionalLineage {
    Coalescence(Lineage),
    OutOfDeme(Lineage),
    OutOfHabitat(Lineage),
}

pub struct SplitExceptionalLineages {
    pub coalescence: Vec<Lineage>,
    pub out_of_deme: Vec<Lineage>,
    pub out_of_habitat: Vec<Lineage>,
}

impl ExceptionalLineage {
    #[must_use]
    pub fn split_vec(exceptional_lineages: Vec<ExceptionalLineage>) -> SplitExceptionalLineages {
        let mut coalescence_lineages = Vec::new();
        let mut out_of_deme_lineages = Vec::new();
        let mut out_of_habitat_lineages = Vec::new();

        for lineage in exceptional_lineages {
            match lineage {
                ExceptionalLineage::Coalescence(lineage) => coalescence_lineages.push(lineage),
                ExceptionalLineage::OutOfDeme(lineage) => out_of_deme_lineages.push(lineage),
                ExceptionalLineage::OutOfHabitat(lineage) => out_of_habitat_lineages.push(lineage),
            }
        }

        SplitExceptionalLineages {
            coalescence: coalescence_lineages,
            out_of_deme: out_of_deme_lineages,
            out_of_habitat: out_of_habitat_lineages,
        }
    }
}
