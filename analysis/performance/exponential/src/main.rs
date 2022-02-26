#![deny(clippy::pedantic)]
#![feature(associated_type_bounds)]

use std::{
    convert::TryFrom,
    num::NonZeroU32,
    sync::atomic::AtomicU64,
    time::{Duration, Instant},
};

use structopt::{
    clap::{Error, ErrorKind},
    StructOpt,
};

use necsim_core::{
    cogs::{Habitat, MathsCore, PrimeableRng, SeedableRng, TurnoverRate},
    landscape::{IndexedLocation, Location},
};
use necsim_core_bond::{NonNegativeF64, OffByOneU32, PositiveF64};
use necsim_core_maths::IntrinsicsMathsCore;
use necsim_impls_no_std::cogs::{
    active_lineage_sampler::independent::event_time_sampler::{
        exp::ExpEventTimeSampler, poisson::PoissonEventTimeSampler, EventTimeSampler,
    },
    habitat::non_spatial::NonSpatialHabitat,
    rng::wyhash::WyHash,
};

use rust_cuda::{
    host::{CudaDropWrapper, LaunchConfig, LaunchPackage, Launcher, TypedKernel},
    rustacuda::{
        context::{Context, ContextFlags},
        device::Device,
        error::CudaResult,
        function::{BlockSize, GridSize},
        stream::{Stream, StreamFlags},
    },
};

use analysis_performance_exponential_kernel::{
    link_exp_kernel, link_poisson_kernel, ExpKernel, ExpKernelArgs, PoissonKernel,
    PoissonKernelArgs, UniformTurnoverRate,
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
        main_gpu(&options);
    } else {
        main_cpu(&options);
    }
}

fn main_cpu(options: &Options) {
    let habitat = NonSpatialHabitat::new(
        (OffByOneU32::one(), OffByOneU32::one()),
        NonZeroU32::new(1).unwrap(),
    );
    let rng = WyHash::<IntrinsicsMathsCore>::seed_from_u64(options.seed);
    let turnover_rate = UniformTurnoverRate::new(options.lambda);
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
    rust_cuda::rustacuda::quick_init().unwrap();

    // Get the first device
    let device = Device::get_device(0).unwrap();

    // Create a context associated to this device
    let _context = Context::create_and_push(ContextFlags::SCHED_AUTO, device).unwrap();

    // Create a stream to submit work to
    let stream = Stream::new(StreamFlags::NON_BLOCKING, None).unwrap();

    let mut total_cycles_sum = AtomicU64::new(0_u64);
    let mut total_time_sum = AtomicU64::new(0_u64);

    match options.mode {
        SamplingMode::Exponential => {
            let mut kernel = BenchmarkExpKernel::try_new(stream, 256.into(), 32.into()).unwrap();

            kernel.benchmark_exp(
                options.seed,
                options.lambda,
                options.delta_t,
                &options.limit.to_le_bytes(),
                &total_cycles_sum,
                &total_time_sum,
            )
        },
        SamplingMode::Poisson => {
            let mut kernel =
                BenchmarkPoissonKernel::try_new(stream, 256.into(), 32.into()).unwrap();

            kernel.benchmark_poisson(
                options.seed,
                options.lambda,
                options.delta_t,
                &options.limit.to_le_bytes(),
                &total_cycles_sum,
                &total_time_sum,
            )
        },
    }
    .unwrap();

    let execution_time = Duration::from_nanos(*total_time_sum.get_mut() / (32 * 256));

    println!(
        "Drawing {} exponential inter-event times with {:?} took {:?} ({}s) [{} cycles].",
        options.limit,
        options.mode,
        execution_time,
        execution_time.as_secs_f64(),
        *total_cycles_sum.get_mut() / (32 * 256),
    );
}

#[allow(clippy::needless_pass_by_value)]
fn sample_exponential_inter_event_times<
    M: MathsCore,
    H: Habitat<M>,
    G: PrimeableRng<M>,
    T: TurnoverRate<M, H>,
    E: EventTimeSampler<M, H, G, T>,
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

pub struct BenchmarkPoissonKernel {
    kernel: TypedKernel<dyn PoissonKernel>,
    stream: CudaDropWrapper<Stream>,
    grid: GridSize,
    block: BlockSize,
    watcher: (),
}

link_poisson_kernel!();

impl BenchmarkPoissonKernel {
    fn try_new(stream: Stream, grid: GridSize, block: BlockSize) -> CudaResult<Self>
    where
        Self: PoissonKernel,
    {
        let stream = CudaDropWrapper::from(stream);
        let kernel = Self::new_kernel()?;

        Ok(Self {
            kernel,
            stream,
            grid,
            block,
            watcher: (),
        })
    }
}

impl Launcher for BenchmarkPoissonKernel {
    type CompilationWatcher = ();
    type KernelTraitObject = dyn PoissonKernel;

    fn get_launch_package(&mut self) -> LaunchPackage<Self> {
        LaunchPackage {
            config: LaunchConfig {
                grid: self.grid.clone(),
                block: self.block.clone(),
                shared_memory_size: 0_u32,
            },

            kernel: &mut self.kernel,
            stream: &mut self.stream,

            watcher: &mut self.watcher,
        }
    }
}

pub struct BenchmarkExpKernel {
    kernel: TypedKernel<dyn ExpKernel>,
    stream: CudaDropWrapper<Stream>,
    grid: GridSize,
    block: BlockSize,
    watcher: (),
}

link_exp_kernel!();

impl BenchmarkExpKernel {
    fn try_new(stream: Stream, grid: GridSize, block: BlockSize) -> CudaResult<Self>
    where
        Self: ExpKernel,
    {
        let stream = CudaDropWrapper::from(stream);
        let kernel = Self::new_kernel()?;

        Ok(Self {
            kernel,
            stream,
            grid,
            block,
            watcher: (),
        })
    }
}

impl Launcher for BenchmarkExpKernel {
    type CompilationWatcher = ();
    type KernelTraitObject = dyn ExpKernel;

    fn get_launch_package(&mut self) -> LaunchPackage<Self> {
        LaunchPackage {
            config: LaunchConfig {
                grid: self.grid.clone(),
                block: self.block.clone(),
                shared_memory_size: 0_u32,
            },

            kernel: &mut self.kernel,
            stream: &mut self.stream,

            watcher: &mut self.watcher,
        }
    }
}
