#![deny(clippy::pedantic)]

#[macro_use]
extern crate contracts;

use std::time::Instant;

use structopt::StructOpt;

use necsim_core::{
    cogs::{Backup, Habitat, PrimeableRng, RngCore, TurnoverRate},
    landscape::{IndexedLocation, Location},
};
use necsim_impls_no_std::cogs::{
    active_lineage_sampler::independent::event_time_sampler::{
        exp::ExpEventTimeSampler, poisson::PoissonEventTimeSampler, EventTimeSampler,
    },
    habitat::non_spatial::NonSpatialHabitat,
    rng::wyhash::WyHash,
};

#[derive(Debug, StructOpt)]
enum SamplingMode {
    Poisson,
    Exponential,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "exponential",
    about = "Generates a stream of exponential inter event times"
)]
struct Options {
    #[structopt(long)]
    seed: u64,
    #[structopt(long)]
    limit: u128,
    #[structopt(long)]
    delta_t: f64,
    #[structopt(long)]
    lambda: f64,
    #[structopt(subcommand)]
    mode: SamplingMode,
}

fn main() {
    let options = Options::from_args();

    let habitat = NonSpatialHabitat::new((1, 1), 1);
    let rng = WyHash::seed_from_u64(options.seed);
    let turnover_rate = UniformTurnoverRate {
        turnover_rate: options.lambda,
    };
    let indexed_location = IndexedLocation::new(Location::new(0, 0), 0);

    match options.mode {
        SamplingMode::Poisson => sample_exponential_inter_event_times(
            habitat,
            rng,
            turnover_rate,
            PoissonEventTimeSampler::new(options.delta_t),
            indexed_location,
            options.limit,
        ),
        SamplingMode::Exponential => sample_exponential_inter_event_times(
            habitat,
            rng,
            turnover_rate,
            ExpEventTimeSampler::new(options.delta_t),
            indexed_location,
            options.limit,
        ),
    }
}

#[allow(clippy::needless_pass_by_value)]
fn sample_exponential_inter_event_times<
    H: Habitat,
    G: PrimeableRng,
    T: TurnoverRate<H>,
    E: EventTimeSampler<H, G, T>,
>(
    habitat: H,
    mut rng: G,
    turnover_rate: T,
    event_time_sampler: E,
    indexed_location: IndexedLocation,
    limit: u128,
) {
    let mut last_event_time = 0.0_f64;

    let start = Instant::now();

    for _ in 0..limit {
        let next_event_time = event_time_sampler.next_event_time_at_indexed_location_after(
            &indexed_location,
            last_event_time,
            &habitat,
            &mut rng,
            &turnover_rate,
        );

        last_event_time = next_event_time;
    }

    let finish = Instant::now();

    println!(
        "Drawing {} exponential inter-event times with {:?} took {:?} ({}s).",
        limit,
        event_time_sampler,
        finish - start,
        (finish - start).as_secs_f64()
    );
}

#[derive(Debug)]
pub struct UniformTurnoverRate {
    turnover_rate: f64,
}

#[contract_trait]
impl Backup for UniformTurnoverRate {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            turnover_rate: self.turnover_rate,
        }
    }
}

#[contract_trait]
impl<H: Habitat> TurnoverRate<H> for UniformTurnoverRate {
    #[must_use]
    #[inline]
    fn get_turnover_rate_at_location(&self, _location: &Location, _habitat: &H) -> f64 {
        self.turnover_rate
    }
}
