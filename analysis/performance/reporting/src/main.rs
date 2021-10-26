#![deny(clippy::pedantic)]
#![feature(associated_type_bounds)]

use std::{convert::TryFrom, marker::PhantomData};

use log::LevelFilter;
use structopt::{
    clap::{Error, ErrorKind},
    StructOpt,
};

use necsim_core_bond::{ClosedUnitF64, NonNegativeF64};
use necsim_core_maths::IntrinsicsMathsCore;
use necsim_impls_std::cogs::rng::pcg::Pcg;

use necsim_core::{
    cogs::{LineageStore, SeedableRng},
    reporter::Reporter,
    simulation::SimulationBuilder,
};
use necsim_impls_no_std::cogs::{
    active_lineage_sampler::classical::ClassicalActiveLineageSampler,
    coalescence_sampler::unconditional::UnconditionalCoalescenceSampler,
    dispersal_sampler::almost_infinite_normal::AlmostInfiniteNormalDispersalSampler,
    emigration_exit::never::NeverEmigrationExit,
    event_sampler::unconditional::UnconditionalEventSampler,
    habitat::almost_infinite::AlmostInfiniteHabitat,
    immigration_entry::never::NeverImmigrationEntry,
    lineage_reference::in_memory::InMemoryLineageReference,
    lineage_store::coherent::globally::almost_infinite::AlmostInfiniteLineageStore,
    origin_sampler::{almost_infinite::AlmostInfiniteOriginSampler, pre_sampler::OriginPreSampler},
    speciation_probability::uniform::UniformSpeciationProbability,
    turnover_rate::uniform::UniformTurnoverRate,
};

use necsim_plugins_common::{
    biodiversity::BiodiversityReporter, event_counter::EventCounterReporter,
    execution_time::ExecutionTimeReporter,
};

mod minimal_logger;
use minimal_logger::MinimalLogger;

#[derive(Debug, StructOpt)]
#[allow(clippy::enum_variant_names)]
enum ReportingMode {
    ProgressOnly,
    ProgressSpeciation,
    ProgressSpeciationDispersal,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "reporting",
    about = "Performs an (almost) infinite simulation with compiled reporting."
)]
struct Options {
    #[structopt(long)]
    seed: u64,
    #[structopt(long)]
    radius: u32,
    #[structopt(long, parse(try_from_str = try_from_str))]
    sigma: NonNegativeF64,
    #[structopt(long, parse(try_from_str = try_from_str))]
    speciation: ClosedUnitF64,
    #[structopt(long, parse(try_from_str = try_from_str))]
    sample: ClosedUnitF64,
    #[structopt(subcommand)]
    mode: ReportingMode,
}

fn try_from_str<T: TryFrom<f64, Error: std::fmt::Display>>(input: &str) -> Result<T, Error> {
    let value: f64 = input
        .parse()
        .map_err(|err| Error::with_description(&format!("{}", err), ErrorKind::ValueValidation))?;

    T::try_from(value)
        .map_err(|err| Error::with_description(&format!("{}", err), ErrorKind::ValueValidation))
}

static MINIMAL_LOGGER: MinimalLogger = MinimalLogger;

fn main() {
    let options = Options::from_args();

    // Set up the minimal logger to stdout/stderr
    log::set_logger(&MINIMAL_LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);

    let progress = ExecutionTimeReporter::default();
    let speciation = BiodiversityReporter::default();
    let dispersal = EventCounterReporter::default();

    match options.mode {
        ReportingMode::ProgressOnly => simulate(&options, necsim_core::ReporterGroup![progress]),
        ReportingMode::ProgressSpeciation => {
            simulate(&options, necsim_core::ReporterGroup![progress, speciation]);
        },
        ReportingMode::ProgressSpeciationDispersal => simulate(
            &options,
            necsim_core::ReporterGroup![progress, speciation, dispersal],
        ),
    }
}

fn simulate<R: Reporter>(options: &Options, mut reporter: R) {
    reporter.initialise().unwrap();

    let habitat = AlmostInfiniteHabitat::default();
    let dispersal_sampler = AlmostInfiniteNormalDispersalSampler::new(options.sigma);
    let turnover_rate = UniformTurnoverRate::default();
    let speciation_probability = UniformSpeciationProbability::new(options.speciation);
    let rng = Pcg::<IntrinsicsMathsCore>::seed_from_u64(options.seed);
    let lineage_store =
        AlmostInfiniteLineageStore::from_origin_sampler(AlmostInfiniteOriginSampler::new(
            OriginPreSampler::all().percentage(options.sample),
            &habitat,
            options.radius,
        ));
    let coalescence_sampler = UnconditionalCoalescenceSampler::default();
    let emigration_exit = NeverEmigrationExit::default();
    let event_sampler = UnconditionalEventSampler::default();
    let immigration_entry = NeverImmigrationEntry::default();
    let active_lineage_sampler = ClassicalActiveLineageSampler::new(&lineage_store);

    let simulation = SimulationBuilder {
        maths: PhantomData::<IntrinsicsMathsCore>,
        habitat,
        lineage_reference: PhantomData::<InMemoryLineageReference>,
        lineage_store,
        dispersal_sampler,
        coalescence_sampler,
        turnover_rate,
        speciation_probability,
        emigration_exit,
        event_sampler,
        active_lineage_sampler,
        rng,
        immigration_entry,
    }
    .build();

    simulation.simulate(&mut reporter);

    reporter.finalise();
}
