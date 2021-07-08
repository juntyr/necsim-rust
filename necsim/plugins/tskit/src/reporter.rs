use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    convert::TryFrom,
    fmt,
    fs::OpenOptions,
    io,
};

use serde::{Deserialize, Serialize, Serializer};
use tskit::{
    provenance::Provenance, TableCollection, TableOutputOptions, TableSortOptions,
    TreeSequenceFlags,
};

use necsim_core::{
    event::{DispersalEvent, SpeciationEvent},
    impl_finalise, impl_report,
    landscape::IndexedLocation,
    lineage::GlobalLineageReference,
    reporter::Reporter,
};
use necsim_core_bond::NonNegativeF64;

const TSK_SEQUENCE_MIN: f64 = 0.0_f64;
const TSK_SEQUENCE_MAX: f64 = 1.0_f64;

#[allow(clippy::module_name_repetitions)]
#[derive(Deserialize)]
#[serde(try_from = "TskitTreeReporterArgs")]
pub struct TskitTreeReporter {
    last_parent_prior_time: Option<(GlobalLineageReference, NonNegativeF64)>,
    last_speciation_event: Option<SpeciationEvent>,
    last_dispersal_event: Option<DispersalEvent>,

    origins: HashMap<GlobalLineageReference, IndexedLocation>,
    children: HashMap<GlobalLineageReference, Vec<(GlobalLineageReference, f64)>>,

    table: TableCollection,

    output: String,
}

impl Serialize for TskitTreeReporter {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        TskitTreeReporterArgs {
            output: self.output.clone(),
        }
        .serialize(serializer)
    }
}

impl fmt::Debug for TskitTreeReporter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("TskitTreeReporter")
            .field("output", &self.output)
            .finish()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename = "TskitTreeReporter")]
struct TskitTreeReporterArgs {
    output: String,
}

impl TryFrom<TskitTreeReporterArgs> for TskitTreeReporter {
    type Error = io::Error;

    fn try_from(args: TskitTreeReporterArgs) -> Result<Self, Self::Error> {
        // Preliminary argument parsing check if the output is a writable file
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&args.output)?;
        std::mem::drop(file);

        let table = TableCollection::new(TSK_SEQUENCE_MAX)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

        Ok(Self {
            last_parent_prior_time: None,
            last_speciation_event: None,
            last_dispersal_event: None,

            origins: HashMap::new(),
            children: HashMap::new(),

            table,

            output: args.output,
        })
    }
}

impl Reporter for TskitTreeReporter {
    impl_report!(speciation(&mut self, speciation: Used) {
        if speciation.prior_time == 0.0_f64 {
            self.store_individual_origin(&speciation.global_lineage_reference, &speciation.origin);
        }

        if Some(speciation) == self.last_speciation_event.as_ref() {
            if let Some((parent, prior_time)) = &self.last_parent_prior_time {
                if prior_time != &speciation.prior_time {
                    let parent = parent.clone();
                    self.store_individual_coalescence(&speciation.global_lineage_reference, parent, speciation.prior_time.get());
                }
            }
        } else {
            self.store_individual_speciation(&speciation.global_lineage_reference, speciation.event_time.get());
        }

        self.last_speciation_event = Some(speciation.clone());
        self.last_parent_prior_time = Some(
            (speciation.global_lineage_reference.clone(), speciation.prior_time)
        );
    });

    impl_report!(dispersal(&mut self, dispersal: Used) {
        if dispersal.prior_time == 0.0_f64 {
            self.store_individual_origin(&dispersal.global_lineage_reference, &dispersal.origin);
        }

        if Some(dispersal) == self.last_dispersal_event.as_ref() {
            if let Some((parent, prior_time)) = &self.last_parent_prior_time {
                if prior_time != &dispersal.prior_time {
                    let parent = parent.clone();
                    self.store_individual_coalescence(&dispersal.global_lineage_reference, parent, dispersal.prior_time.get());
                }
            }
        } else if let Some(parent) = dispersal.interaction.parent() {
            self.store_individual_coalescence(&dispersal.global_lineage_reference, parent, dispersal.event_time.get());
        }

        self.last_dispersal_event = Some(dispersal.clone());
        self.last_parent_prior_time = Some(
            (dispersal.global_lineage_reference.clone(), dispersal.prior_time)
        );
    });

    impl_report!(progress(&mut self, _progress: Ignored) {});

    impl_finalise!((mut self) {
        self.table.full_sort(TableSortOptions::NONE).unwrap();

        self.table.tree_sequence(TreeSequenceFlags::BUILD_INDEXES).unwrap().dump(&self.output, TableOutputOptions::NONE).unwrap();
    });

    fn initialise(&mut self) -> Result<(), String> {
        let provenance =
            crate::provenance::TskitProvenance::try_new().map_err(|err| err.to_string())?;
        let provenance_json = serde_json::to_string(&provenance).map_err(|err| err.to_string())?;

        self.table
            .add_provenance(&provenance_json)
            .map_err(|err| err.to_string())
            .map(|_| ())
    }
}

impl crate::reporter::TskitTreeReporter {
    fn store_individual_origin(
        &mut self,
        reference: &GlobalLineageReference,
        location: &IndexedLocation,
    ) {
        self.origins.insert(reference.clone(), location.clone());
    }

    fn store_individual_speciation(&mut self, reference: &GlobalLineageReference, time: f64) {
        let parent_id = if let Some(origin) = self.origins.remove(reference) {
            self.table
                .add_individual(
                    0_u32,
                    &[
                        f64::from(origin.location().x()),
                        f64::from(origin.location().y()),
                        f64::from(origin.index()),
                    ],
                    &[],
                )
                .unwrap()
        } else {
            return;
        };

        let parent_node_id = self
            .table
            .add_node(tskit::TSK_NODE_IS_SAMPLE, time, tskit::TSK_NULL, parent_id)
            .unwrap();

        let mut stack = VecDeque::from(vec![(reference.clone(), parent_id, parent_node_id)]);

        while let Some((parent, parent_id, parent_node_id)) = stack.pop_front() {
            if let Some(children) = self.children.remove(&parent) {
                for (child, time) in children {
                    if let Some(origin) = self.origins.remove(&child) {
                        let child_id = self
                            .table
                            .add_individual(
                                0_u32,
                                &[
                                    f64::from(origin.location().x()),
                                    f64::from(origin.location().y()),
                                    f64::from(origin.index()),
                                ],
                                &[parent_id],
                            )
                            .unwrap();

                        let child_node_id = self
                            .table
                            .add_node(tskit::TSK_NODE_IS_SAMPLE, time, tskit::TSK_NULL, child_id)
                            .unwrap();

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

    fn store_individual_coalescence(
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
}
