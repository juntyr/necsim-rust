use std::collections::{hash_map::Entry, VecDeque};

use tskit::{provenance::Provenance, TableOutputOptions, TableSortOptions, TreeSequenceFlags};

use necsim_core::{landscape::IndexedLocation, lineage::GlobalLineageReference};

use super::{
    metadata::GlobalLineageMetadata, TskitTreeReporter, TSK_SEQUENCE_MAX, TSK_SEQUENCE_MIN,
};

impl TskitTreeReporter {
    pub(super) fn store_individual_origin(
        &mut self,
        reference: &GlobalLineageReference,
        location: &IndexedLocation,
    ) {
        self.origins.insert(reference.clone(), location.clone());
    }

    pub(super) fn store_individual_speciation(
        &mut self,
        parent: &GlobalLineageReference,
        time: f64,
    ) {
        // Insert the speciating parent lineage as an individual
        let parent_id = if let Some(origin) = self.origins.remove(parent) {
            self.table
                .add_individual_with_metadata(
                    0_u32,
                    &[
                        f64::from(origin.location().x()),
                        f64::from(origin.location().y()),
                        f64::from(origin.index()),
                    ],
                    &[],
                    Some(GlobalLineageMetadata::new(parent)),
                )
                .unwrap()
        } else {
            return;
        };

        // Create the speciation node
        let parent_node_id = self
            .table
            .add_node_with_metadata(
                tskit::TSK_NODE_IS_SAMPLE,
                time,
                tskit::TSK_NULL,
                parent_id,
                Some(GlobalLineageMetadata::new(parent)),
            )
            .unwrap();

        let mut stack = VecDeque::from(vec![(parent.clone(), parent_id, parent_node_id)]);

        // Iteratively insert the parent's successors in breadth first order
        while let Some((parent, parent_id, parent_node_id)) = stack.pop_front() {
            if let Some(children) = self.children.remove(&parent) {
                for (child, time) in children {
                    if let Some(origin) = self.origins.remove(&child) {
                        // Insert the coalesced child lineage as an individual
                        let child_id = self
                            .table
                            .add_individual_with_metadata(
                                0_u32,
                                &[
                                    f64::from(origin.location().x()),
                                    f64::from(origin.location().y()),
                                    f64::from(origin.index()),
                                ],
                                &[parent_id],
                                Some(GlobalLineageMetadata::new(&child)),
                            )
                            .unwrap();

                        // Create the coalescence node
                        let child_node_id = self
                            .table
                            .add_node_with_metadata(
                                tskit::TSK_NODE_IS_SAMPLE,
                                time,
                                tskit::TSK_NULL,
                                child_id,
                                Some(GlobalLineageMetadata::new(&child)),
                            )
                            .unwrap();

                        // Add the parent-child relation between the nodes
                        self.table
                            .add_edge(
                                TSK_SEQUENCE_MIN,
                                TSK_SEQUENCE_MAX,
                                parent_node_id,
                                child_node_id,
                            )
                            .unwrap();

                        stack.push_back((child, child_id, child_node_id));
                    }
                }
            }
        }
    }

    pub(super) fn store_individual_coalescence(
        &mut self,
        child: &GlobalLineageReference,
        parent: GlobalLineageReference,
        time: f64,
    ) {
        match self.children.entry(parent) {
            Entry::Occupied(mut entry) => entry.get_mut().push((child.clone(), time)),
            Entry::Vacant(entry) => {
                entry.insert(vec![(child.clone(), time)]);
            },
        }
    }

    pub(super) fn store_provenance(&mut self) -> Result<(), String> {
        // Capture and record the provenance information inside the table
        let provenance =
            crate::provenance::TskitProvenance::try_new().map_err(|err| err.to_string())?;
        let provenance_json = serde_json::to_string(&provenance).map_err(|err| err.to_string())?;

        self.table
            .add_provenance(&provenance_json)
            .map_err(|err| err.to_string())
            .map(|_| ())
    }

    pub(super) fn output_tree_sequence(mut self) {
        self.table.full_sort(TableSortOptions::NONE).unwrap();

        // Output the tree sequence to the specified `output` file
        self.table
            .tree_sequence(TreeSequenceFlags::BUILD_INDEXES)
            .unwrap()
            .dump(&self.output, TableOutputOptions::NONE)
            .unwrap();
    }
}
