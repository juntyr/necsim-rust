use std::collections::VecDeque;

use necsim_core_bond::NonNegativeF64;
use tskit::{IndividualId, NodeId, TableOutputOptions, TableSortOptions, TreeSequenceFlags};

use necsim_core::{landscape::IndexedLocation, lineage::GlobalLineageReference};

use super::{
    metadata::GlobalLineageMetadata, TskitTreeReporter, TSK_SEQUENCE_MAX, TSK_SEQUENCE_MIN,
};

const TSK_FLAGS_EMPTY: tskit::tsk_flags_t = 0_u32;

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
        time: NonNegativeF64,
    ) {
        // Resolve the actual parent, irrespective of duplicate individuals
        let mut parent = parent;
        while let Some(parent_parent) = self.parents.get(parent) {
            parent = parent_parent;
        }
        let parent = parent.clone();

        // Insert the speciating parent lineage, then store its successors, too
        if let Some((parent_individual, parent_node)) = self.store_lineage(&parent, time, None) {
            self.store_children_of_parent(&parent, parent_individual, parent_node);
        }
    }

    pub(super) fn store_individual_coalescence(
        &mut self,
        child: &GlobalLineageReference,
        parent: &GlobalLineageReference,
        time: NonNegativeF64,
    ) {
        // Resolve the actual child, irrespective of duplicate individuals
        let mut child = child;
        while let Some(child_parent) = self.parents.get(child) {
            child = child_parent;
        }
        let child = child.clone();

        // Resolve the actual parent, irrespective of duplicate individuals
        let mut parent = parent;
        while let Some(parent_parent) = self.parents.get(parent) {
            parent = parent_parent;
        }
        let parent = parent.clone();

        self.parents.insert(child.clone(), parent.clone());

        if let Some((parent_individual, parent_node)) = self.tskit_ids.get(&parent).copied() {
            // The parent has already been inserted
            //  -> immediately store child and its successors
            if let Some((child_individual, child_node)) =
                self.store_lineage(&child, time, Some((parent_individual, parent_node)))
            {
                self.store_children_of_parent(&child, child_individual, child_node);
            }
        } else {
            // The parent has not been inserted yet
            //  -> postpone insertion and remember the child
            self.children.entry(parent).or_default().push((child, time));
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

impl TskitTreeReporter {
    /// Store a lineage as a `tskit` individual and birth node, optionally with
    /// a parent relationship
    fn store_lineage(
        &mut self,
        reference: &GlobalLineageReference,
        time: NonNegativeF64,
        parent: Option<(IndividualId, NodeId)>,
    ) -> Option<(IndividualId, NodeId)> {
        let origin = self.origins.remove(reference)?;
        let location = [
            f64::from(origin.location().x()),
            f64::from(origin.location().y()),
            f64::from(origin.index()),
        ];
        let metadata = GlobalLineageMetadata::new(reference);
        let parents = if let Some((parent_individual, _parent_node)) = &parent {
            std::slice::from_ref(parent_individual)
        } else {
            &[]
        };

        // Insert the lineage as an individual
        let individual_id = self
            .table
            .add_individual_with_metadata(TSK_FLAGS_EMPTY, location, parents, metadata)
            .unwrap();

        // Create corresponding node
        let node_id = self
            .table
            .add_node_with_metadata(
                tskit::TSK_NODE_IS_SAMPLE,
                time.get(),
                tskit::PopulationId::NULL,
                individual_id,
                metadata,
            )
            .unwrap();

        if let Some((_parent_individual, parent_node)) = parent {
            // Add the parent-child relation between the nodes
            self.table
                .add_edge(TSK_SEQUENCE_MIN, TSK_SEQUENCE_MAX, parent_node, node_id)
                .unwrap();
        }

        // Store the individual and node for potential late coalescences
        self.tskit_ids
            .insert(reference.clone(), (individual_id, node_id));

        Some((individual_id, node_id))
    }

    /// Store all the children lineages of the parent lineage
    ///  as `tskit` individuals with birth nodes
    fn store_children_of_parent(
        &mut self,
        parent: &GlobalLineageReference,
        parent_individual: IndividualId,
        parent_node: NodeId,
    ) {
        let mut stack = VecDeque::from(vec![(parent.clone(), parent_individual, parent_node)]);

        // Iteratively insert the parent's successors in breadth first order
        while let Some((parent, parent_individual, parent_node)) = stack.pop_front() {
            if let Some(children) = self.children.remove(&parent) {
                for (child, time) in children {
                    // Insert the coalesced child lineage
                    if let Some((child_individual, child_node)) =
                        self.store_lineage(&child, time, Some((parent_individual, parent_node)))
                    {
                        stack.push_back((child, child_individual, child_node));
                    }
                }
            }
        }
    }
}
