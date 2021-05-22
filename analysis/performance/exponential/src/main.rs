#![deny(clippy::pedantic)]
#![feature(associated_type_bounds)]

#[macro_use]
extern crate contracts;

use std::{
    convert::TryFrom,
    time::{Duration, Instant},
};

use necsim_core_bond::{NonNegativeF64, PositiveF64};
use structopt::{
    clap::{Error, ErrorKind},
    StructOpt,
};

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
    #[structopt(long, parse(try_from_str = try_from_str))]
    delta_t: PositiveF64,
    #[structopt(long, parse(try_from_str = try_from_str))]
    lambda: PositiveF64,
    #[structopt(long)]
    cuda: bool,
    #[structopt(subcommand)]
    mode: SamplingMode,
}

fn try_from_str<T: TryFrom<f64, Error: std::fmt::Display>>(input: &str) -> Result<T, Error> {
    let value: f64 = input
        .parse()
        .map_err(|err| Error::with_description(&format!("{}", err), ErrorKind::ValueValidation))?;

    T::try_from(value)
        .map_err(|err| Error::with_description(&format!("{}", err), ErrorKind::ValueValidation))
}

fn main() {
    let options = Options::from_args();

    if options.cuda {
        main_gpu(&options)
    } else {
        main_cpu(&options)
    }
}

fn main_cpu(options: &Options) {
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

fn main_gpu(options: &Options) {
    use rust_cuda::common::{DeviceBoxConst, DeviceBoxMut};
    use rustacuda::{launch, memory::DeviceBox, prelude::*};
    use std::ffi::CString;

    rustacuda::quick_init().unwrap();

    // Get the first device
    let device = Device::get_device(0).unwrap();

    // Create a context associated to this device
    let _context = Context::create_and_push(ContextFlags::SCHED_AUTO, device).unwrap();

    // Load the module containing the function we want to call
    let module_data = CString::new(include_str!(env!("CUDA_PTX_KERNEL"))).unwrap();
    let module = Module::load_from_string(&module_data).unwrap();

    // Create a stream to submit work to
    let stream = Stream::new(StreamFlags::NON_BLOCKING, None).unwrap();

    let limit = DeviceBox::new(&options.limit).unwrap();

    let mut total_cycles_sum = DeviceBox::new(&0_u64).unwrap();
    let mut total_time_sum = DeviceBox::new(&0_u64).unwrap();

    match options.mode {
        SamplingMode::Exponential => unsafe {
            launch!(module.benchmark_exp<<<256, 32, 0, stream>>>(
                options.seed,
                options.lambda,
                options.delta_t,
                DeviceBoxConst::from(&limit),
                DeviceBoxMut::from(&mut total_cycles_sum),
                DeviceBoxMut::from(&mut total_time_sum)
            ))
            .unwrap()
        },
        SamplingMode::Poisson => unsafe {
            launch!(module.benchmark_poisson<<<256, 32, 0, stream>>>(
                options.seed,
                options.lambda,
                options.delta_t,
                DeviceBoxConst::from(&limit),
                DeviceBoxMut::from(&mut total_cycles_sum),
                DeviceBoxMut::from(&mut total_time_sum)
            ))
            .unwrap()
        },
    }

    // The kernel launch is asynchronous, so we wait for the kernel to finish
    // executing
    stream.synchronize().unwrap();

    let mut result_total_cycles_sum = 0_u64;
    let mut result_total_time_sum = 0_u64;

    total_cycles_sum
        .copy_to(&mut result_total_cycles_sum)
        .unwrap();
    total_time_sum.copy_to(&mut result_total_time_sum).unwrap();

    let execution_time = Duration::from_nanos(result_total_time_sum / (32 * 256));

    println!(
        "Drawing {} exponential inter-event times with {:?} took {:?} ({}s) [{} cycles].",
        options.limit,
        options.mode,
        execution_time,
        execution_time.as_secs_f64(),
        result_total_cycles_sum / (32 * 256),
    );
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
    let mut last_event_time = NonNegativeF64::zero();

    let start = Instant::now();

    for _ in 0..limit {
        let next_event_time = event_time_sampler.next_event_time_at_indexed_location_weakly_after(
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
    turnover_rate: PositiveF64,
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
    fn get_turnover_rate_at_location(&self, _location: &Location, _habitat: &H) -> NonNegativeF64 {
        // Use a volatile read to ensure that the turnover rate cannot be
        //  optimised out of this benchmark test

        unsafe { core::ptr::read_volatile(&self.turnover_rate) }.into()
    }
}
