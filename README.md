# necsim-rust

[![Gitpod Ready-to-Code](https://img.shields.io/badge/Gitpod-ready--to--code-blue?logo=gitpod)](https://gitpod.io/#https://github.com/MomoLangenstein/necsim-rust)

## Introduction

necsim-rust is a Rust reimplementation of the C++ library [necsim](https://bitbucket.org/thompsonsed/necsim) and its Python wrapper [pycoalescence](https://bitbucket.org/thompsonsed/pycoalescence), which are used to run neutral coalescence biodiversity simulations.

necsim-rust aims to provide a smaller, more concise subset of the functionality of necsim and pycoalescence but be easier to use and extend. For instance, necsim-rust contains the classical coalescence algorithm. Additionally, it implements two Gillespie-based algorithms and a novel independent algorithm with a CPU and a CUDA variant. Furthermore, necsim-rust can use MPI to parallelise the simulation.

necsim-rust is built in a modular way to reduce code duplication and allow the user (and other programmers) to plug together different components to customise the simulated scenario, the algorithm it is simulated with as well as finer implementation details. Currently, necsim-rust supports four built-in scenarios:
- spatially-explicit simulation
- non-spatial simulation
- spatially-implicit simulation with migration from a non-spatial metacommunity to a non-spatial local community
- (almost) infinite simulation with Gaussian Normal dispersal

## Installation

First, you need to clone the necsim-rust GitHub repository:
```shell
> git clone https://github.com/MomoLangenstein/necsim-rust.git
```
necsim-rust is written in the [Rust language](https://www.rust-lang.org/tools/install), which must be installed in your `PATH` first. Next, please run the `setup.sh` command to compile and set up the custom `rustcoalescence-linker`:
```shell
> cd necsim-rust
> ./setup.sh
```
necsim-rust includes a `rust-toolchain` file that configures Rust to use a working nightly toolchain version and install all components required for compilation.

If you also want to use the CUDA-based algorithm, it is **required** that you also install the following:
```shell
> cargo install ptx-linker -f
```

If you also want to use the MPI-based parallelisation, it is *recommended* that you also install the following:
```shell
> cargo install cargo-mpirun -f
```

## Project structure

necsim-rust consists of the following crates:
- necsim/: this folder contains the core declaration of the simulation and implementation of its components
    - core/: `necsim-core` declares the core structs, simulation cogs traits, as well as the generic `Simulation`.
        - bond/: `necsim-core-bond` declares helper data types which guarantee a limited value range and are used for encoding preconditions through data types.
    - impls/:
        - no-std/: `necsim-impls-no-std` contains the implementations of cogs that **do not** require the Rust standard library
        - std/: `necsim-impls-std` contains the implementations of cogs that **do** require the Rust standard library
        - cuda/: `necsim-impls-cuda` contains the implementations of CUDA specific cogs
        - mpi/: `necsim-impls-mpi` contains the implementation of the MPI-based parallelisation backend
    - plugins/:
        - core/: `necsim-plugins-core` implements the reporter plugin system and provides the functionality to export and load plugins
        - common/: `necsim-plugins-common` implements common analysis reporters, e.g. to measure biodiversity, print a progress bar, etc.
        - metacommunity/: `necsim-plugins-metacommunity` implements a reporter which measures migrations to a static external metacommunity, which can be simulated separately using the non-spatial scenario
        - csv/: `necsim-plugins-csv` implements a reporter which records events in a CSV file
        - coalescence/: `necsim-plugins-coalescence` is **WORK IN PROGRESS**
- rustcoalescence/: `rustcoalescence` provides the command-line interface.
    - linker/: `rustcoalescence-linker` is a custom linker used during the compilation.
    - scenarios/: `rustcoalescence-scenarios` contains the glue code to put together the cogs for the built-in scenarios. It is specifically built only for reducing code duplication in rustcoalescence, not for giving a minimal example of how to construct a simulation.
    - algorithms/:
        - monolithic/: `rustcoalescence-algorithms-monlithic` contains the glue code to put together the cogs for the three **monolithic** coalescence algorithms. It is specifically built only for reducing code duplication in rustcoalescence, not for giving a minimal example of how to construct a simulation.
            - src/classical: `ClassicalAlgorithm` is a good allrounder that approximates exponential inter-event times with a Geometric distribution and only supports uniform turnover rates
            - src/gillespie: `GillespieAlgorithm` is a mathematically correct Gillespie-algorithm-based implementation
            - src/skipping_gillespie: `SkippingGillespieAlgorithm` is a mathematically correct Gillespie-algorithm-based implementation which skips self-dispersal events without coalescence. Therefore, it is very fast on habitats with high self-dispersal probabilities.
        - independent/: `rustcoalescence-algorithms-independent` contains the glue code to put together the cogs for the **independent** coalescence algorithm on the CPU. The algorithm treats the simulation as embarrassingly parallel problem and can be used to simulate subdomains of the simulation separately and piece the results back afterwards without loss of consistency.
        - cuda/: `rustcoalescence-algorithms-cuda` contains the glue code to put together the cogs for the **independent** coalescence algorithm on a CUDA 3.5 capable GPU. The algorithm treats the simulation as embarrassingly parallel problem and can be used to simulate subdomains of the simulation separately and piece the results back afterwards without loss of consistency.
- rust-cuda/: `rust-cuda` provides automatically derivable traits to add more (but not full) type safety to sharing data structures between the CPU and GPU
- third-party/: this folder contains multiple third-party crates which had to be modified (mostly to enable `no-std` support)

## Compilation

To compile `rustcoalescence`, you need to decide which algorithms you want to compile with it. You can enable any of the four provided algorithms by enabling its corresponding feature of the same name. For instance, to compile all CPU-based algorithms, you can use
```shell
> cargo build --release --features rustcoalescence-algorithms-monolithic --features rustcoalescence-algorithms-independent
```
To compile with CUDA support, you first need to ensure that the dynamic CUDA libraries are in the `LD_LIBRARY_PATH` and enable the `necsim-cuda` feature:
```shell
> LIBRARY_PATH="$LD_LIBRARY_PATH" cargo build --release [...] --features rustcoalescence-algorithms-cuda
```
To compile with MPI support, you need to enable the `necsim-partitioning-mpi` feature:
```shell
> cargo build --release [...] --features necsim-partitioning-mpi
```
After compilation, you can then run `rustcoalescence` using:
```
> cargo run --release --features [...]
```
or directly call the binary at:
```
> target/release/rustcoalescence
```
If you have installed `cargo mpirun`, you can also use it as a wrapper:
```
> cargo mpirun --release --bin rustcoalescence --features [...] -n <NUMBER_PARTITIONS> --
```

## Running rustcoalescence

`rustcoalescence` has two subcommands: `simulate` and `replay` and accepts command-line arguments in the following format:
```shell
> rustcoalescence <SUBCOMMAND> args..
```
Here, `args..` is a configuration string in [RON](https://github.com/ron-rs/ron) format, which can also directly be read from a configuration file:
```shell
> rustcoalescence <SUBCOMMAND> $(<config.ron)
```
Please refer to [docs/simulate.ron](docs/simulate.ron) and [docs/replay.ron](docs/replay.ron) for a detailed description of all configuration options. [./simulate.ron](simulate.ron) and [./replay.ron](replay.ron) also provide example configurations.

## GDAL GeoTiff compatibility

pycoalescence and necsim both used GDAL to load habitat and dispersal maps. As rustcoalescence is more strict about type checking the TIFF files, you can use the following commands to convert and compress your GeoTIFF files:
```shell
> gdalwarp -ot Uint32 -co "COMPRESS=LZW" -dstnodata 0 -to "SRC_METHOD=NO_GEOTRANSFORM" input_habitat.tif output_habitat.tif
> gdalwarp -ot Float64 -co "COMPRESS=LZW" -dstnodata 0 -to "SRC_METHOD=NO_GEOTRANSFORM" input_dispersal.tif output_dispersal.tif
```
