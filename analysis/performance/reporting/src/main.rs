#![deny(clippy::pedantic)]
#![feature(associated_type_bounds)]

use std::{convert::TryFrom, marker::PhantomData};

use log::LevelFilter;
use necsim_core_bond::{ClosedUnitF64, NonNegativeF64};
use necsim_impls_std::cogs::rng::pcg::Pcg;
use structopt::{
    clap::{Error, ErrorKind},
    StructOpt,
};

use necsim_core::{
    cogs::{LineageStore, RngCore},
    reporter::Reporter,
    simulation::Simulation,
};
use necsim_impls_no_std::cogs::{
    active_lineage_sampler::classical::ClassicalActiveLineageSampler,
    coalescence_sampler::unconditional::UnconditionalCoalescenceSampler,
    dispersal_sampler::almost_infinite_normal::AlmostInfiniteNormalDispersalSampler,
    emigration_exit::never::NeverEmigrationExit,
    event_sampler::unconditional::UnconditionalEventSampler,
    habitat::almost_infinite::AlmostInfiniteHabitat,
    immigration_entry::never::NeverImmigrationEntry,
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
            simulate(&options, necsim_core::ReporterGroup![progress, speciation])
        },
        ReportingMode::ProgressSpeciationDispersal => simulate(
            &options,
            necsim_core::ReporterGroup![progress, speciation, dispersal],
        ),
    }
}

fn simulate<R: Reporter>(options: &Options, mut reporter: R) {
    let habitat = AlmostInfiniteHabitat::default();
    let dispersal_sampler = AlmostInfiniteNormalDispersalSampler::new(options.sigma);
    let turnover_rate = UniformTurnoverRate::default();
    let speciation_probability = UniformSpeciationProbability::new(options.speciation);
    let rng = Pcg::seed_from_u64(options.seed);
    let lineage_store =
        AlmostInfiniteLineageStore::from_origin_sampler(AlmostInfiniteOriginSampler::new(
            OriginPreSampler::all().percentage(options.sample.get()),
            &habitat,
            options.radius,
        ));
    let coalescence_sampler = UnconditionalCoalescenceSampler::default();
    let emigration_exit = NeverEmigrationExit::default();
    let event_sampler = UnconditionalEventSampler::default();
    let immigration_entry = NeverImmigrationEntry::default();
    let active_lineage_sampler = ClassicalActiveLineageSampler::new(&lineage_store);

    let simulation = Simulation::builder()
        .habitat(habitat)
        .rng(rng)
        .speciation_probability(speciation_probability)
        .dispersal_sampler(dispersal_sampler)
        .lineage_reference(PhantomData)
        .lineage_store(lineage_store)
        .emigration_exit(emigration_exit)
        .coalescence_sampler(coalescence_sampler)
        .turnover_rate(turnover_rate)
        .event_sampler(event_sampler)
        .immigration_entry(immigration_entry)
        .active_lineage_sampler(active_lineage_sampler)
        .build();

    reporter.initialise().unwrap();
    simulation.simulate(&mut reporter);
    reporter.finalise();
}
