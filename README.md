# necsim-rust &emsp; [![CI Status]][workflow] [![Rust Doc]][docs] [![License Status]][fossa] [![Code Coverage]][codecov] [![Binder Demo]][binder] [![Gitpod Ready-to-Code]][gitpod] [![DOI]][zenodo]

[CI Status]: https://img.shields.io/github/workflow/status/MomoLangenstein/necsim-rust/CI/main?label=CI
[workflow]: https://github.com/MomoLangenstein/necsim-rust/actions/workflows/ci.yml?query=branch%3Amain

[Rust Doc]: https://img.shields.io/badge/docs-main-blue
[docs]: https://momolangenstein.github.io/necsim-rust/

[License Status]: https://app.fossa.com/api/projects/git%2Bgithub.com%2FMomoLangenstein%2Fnecsim-rust.svg?type=shield
[fossa]: https://app.fossa.com/projects/git%2Bgithub.com%2FMomoLangenstein%2Fnecsim-rust?ref=badge_shield

[Code Coverage]: https://img.shields.io/codecov/c/github/MomoLangenstein/necsim-rust?token=DCT0WVLU5V
[codecov]: https://codecov.io/gh/MomoLangenstein/necsim-rust

[Binder Demo]: https://img.shields.io/badge/binder-demo-579ACA.svg?logo=data:image/png;base64,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
[binder]: https://mybinder.org/v2/gh/MomoLangenstein/necsim-rust.git/demo?filepath=demo.ipynb

[Gitpod Ready-to-Code]: https://img.shields.io/badge/Gitpod-ready-blue?logo=gitpod
[gitpod]: https://gitpod.io/#https://github.com/MomoLangenstein/necsim-rust

[DOI]: https://zenodo.org/badge/303474757.svg
[zenodo]: https://zenodo.org/badge/latestdoi/303474757

## Introduction

necsim-rust is a Rust reimplementation of the C++ library [necsim](https://bitbucket.org/thompsonsed/necsim) and its Python wrapper [pycoalescence](https://bitbucket.org/thompsonsed/pycoalescence), which are used to run neutral coalescence biodiversity simulations.

necsim-rust aims to provide a smaller, more concise subset of the functionality of necsim and pycoalescence but be easier to use and extend. For instance, necsim-rust contains the classical coalescence algorithm. Additionally, it implements two Gillespie-based algorithms and a novel independent algorithm with a CPU and a CUDA variant. Furthermore, necsim-rust can use MPI to parallelise the simulation.

necsim-rust is built in a modular way to reduce code duplication and allow the user (and other programmers) to plug together different components to customise the simulated scenario, the algorithm it is simulated with as well as finer implementation details. Currently, necsim-rust supports four built-in scenarios:
- non-spatial model
- spatially implicit model with migration from a non-spatial metacommunity to a non-spatial local community
- spatially explicit (almost) infinite model with Gaussian Normal dispersal
- spatially-explicit simulation with habitat and dispersal maps

## Prerequisites

First, you need to clone the necsim-rust GitHub repository:
```shell
> git clone https://github.com/MomoLangenstein/necsim-rust.git
```
necsim-rust is written in the [Rust language](https://www.rust-lang.org/tools/install), which must be installed in your `PATH` first. necsim-rust includes a `rust-toolchain` file that configures Rust to use a working nightly toolchain version and install all components required for compilation. If you want to use necsim-rust on a target different than `x86_64-unknown-linux-gnu`, please update [rust-toolchain](rust-toolchain) and [.cargo/config](.cargo/config) config files accordingly.

If you also want to use the CUDA-based algorithm, it is **required** that you also install the following:
```shell
> cargo install ptx-linker -f
```

## Installation

To install `rustcoalescence`, you need to decide which algorithms you want to compile with it. You can enable the provided algorithms by enabling their corresponding features. For instance, to compile all CPU-based algorithms, you can use
```shell
> cargo install --path rustcoalescence --locked --features rustcoalescence-algorithms-monolithic --features rustcoalescence-algorithms-independent
```
To install with CUDA support, you first need to ensure that the dynamic CUDA libraries are in the `LD_LIBRARY_PATH` and enable the `rustcoalescence-algorithms-cuda` feature:
```shell
> LIBRARY_PATH="$LD_LIBRARY_PATH" cargo install --path rustcoalescence --locked [...] --features rustcoalescence-algorithms-cuda
```
To compile with MPI support, you need to enable the `necsim-partitioning-mpi` feature:
```shell
> cargo install --path rustcoalescence --locked [...] --features necsim-partitioning-mpi
```
After compilation, you can then run `rustcoalescence` using:
```shell
> rustcoalescence [...]
```
If you want to use any of the provided reporter analysis plugins, you have to compile them manually. For instance, to compile the `common` plugin which includes the `Biodiversity()`, `Counter()`, `Execution()`, `Progress()` and `Verbose()` reporters, you can run:
```shell
> cargo build --release --manifest-path necsim/plugins/common/Cargo.toml
```

## Compiling for Development

If you want to compile the library for development, you can use any of the above installation commands, but replace
```shell
> cargo install --path rustcoalescence --locked [...]
```
with
```shell
> cargo build --release [...]
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

## Project structure

necsim-rust consists of the following crates:
- necsim/: this folder contains the core declaration of the simulation and implementation of its components
    - core/: `necsim-core` declares the core structs, simulation cogs traits, as well as the generic `Simulation`.
        - bond/: `necsim-core-bond` declares helper data types which guarantee a limited value range and are used for encoding preconditions through data types.
    - impls/:
        - no-std/: `necsim-impls-no-std` contains the implementations of cogs that **do not** require the Rust standard library
        - std/: `necsim-impls-std` contains the implementations of cogs that **do** require the Rust standard library
        - cuda/: `necsim-impls-cuda` contains the implementations of CUDA specific cogs
    - plugins/:
        - core/: `necsim-plugins-core` implements the reporter plugin system and provides the functionality to export and load plugins
        - common/: `necsim-plugins-common` implements common analysis reporters, e.g. to measure biodiversity, print a progress bar, etc.
        - metacommunity/: `necsim-plugins-metacommunity` implements a reporter which measures migrations to a static external metacommunity, which can be simulated separately using the non-spatial scenario
        - csv/: `necsim-plugins-csv` implements a reporter which records events in a CSV file
        - species/: `necsim-plugins-species` produces an SQLite database which lists the parent-child relationships of all simulated individuals as well as their species
    - partitioning/:
        - core/: `necsim-partitioning-core` declares the core partitioning traits
        - monolithic/: `necsim-partitioning-monolithic` implements monolithic, i.e. non-parallel partitioning
        - mpi/: `necsim-partitioning-mpi` implements the MPI-based partitioning backend
- rustcoalescence/: `rustcoalescence` provides the command-line interface.
    - linker/: `rustcoalescence-linker` is a custom linker used during the compilation.
    - scenarios/: `rustcoalescence-scenarios` contains the glue code to put together the cogs for the built-in scenarios. It is specifically built only for reducing code duplication in rustcoalescence, not for giving a minimal example of how to construct a simulation.
    - algorithms/:
        - monolithic/: `rustcoalescence-algorithms-monolithic contains the glue code to put together the cogs for the three **monolithic** coalescence algorithms. It is specifically built only for reducing code duplication in rustcoalescence, not for giving a minimal example of how to construct a simulation.
            - src/classical: `ClassicalAlgorithm` is a good allrounder that approximates exponential inter-event times with a Geometric distribution and only supports uniform turnover rates
            - src/gillespie: `GillespieAlgorithm` is a mathematically correct Gillespie-algorithm-based implementation
            - src/skipping_gillespie: `SkippingGillespieAlgorithm` is a mathematically correct Gillespie-algorithm-based implementation that skips self-dispersal events without coalescence. Therefore, it is very fast on habitats with high self-dispersal probabilities.
        - independent/: `rustcoalescence-algorithms-independent` contains the glue code to put together the cogs for the **independent** coalescence algorithm on the CPU. The algorithm treats the simulation as an embarrassingly parallel problem. It can also be used to simulate subdomains of the simulation separately and piece the results back afterwards without loss of consistency.
        - cuda/: `rustcoalescence-algorithms-cuda` contains the glue code to put together the cogs for the **independent** coalescence algorithm on a CUDA 3.5 capable GPU. The algorithm treats the simulation as an embarrassingly parallel problem. It can also be used to simulate subdomains of the simulation separately and piece the results back afterwards without loss of consistency.
- rust-cuda/: `rust-cuda` provides automatically derivable traits to add more (but not complete) type safety to sharing data structures between the CPU and GPU

## GDAL GeoTiff compatibility

pycoalescence and necsim both used GDAL to load habitat and dispersal maps. As rustcoalescence is more strict about type checking the TIFF files, you can use the following commands to convert and compress your GeoTIFF files:
```shell
> gdalwarp -ot Uint32 -co "COMPRESS=LZW" -dstnodata 0 -to "SRC_METHOD=NO_GEOTRANSFORM" -to "DST_METHOD=NO_GEOTRANSFORM" input_habitat.tif output_habitat.tif
> gdalwarp -ot Float64 -co "COMPRESS=LZW" -dstnodata 0 -to "SRC_METHOD=NO_GEOTRANSFORM" -to "DST_METHOD=NO_GEOTRANSFORM" input_dispersal.tif output_dispersal.tif
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
