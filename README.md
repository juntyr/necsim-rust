# necsim-rust

## Introduction

necsim-rust is a Rust reimplementation of the C++ library [necsim](https://bitbucket.org/thompsonsed/necsim) and its Python wrapper [pycoalescence](https://bitbucket.org/thompsonsed/pycoalescence), which are used to run neutral coalescence biodiversity simulations.

necsim-rust aims to provide a smaller, more concise subset of the functionality of necsim and pycoalescence, but be easier to use and extend. For instance, necsim-rust contains the classical coalescence algorithm and two variants based on the Gillespie algorithm and a CUDA-based implementation. In the future, we will add another algorithm variant to enable MPI-based parallelisation by splitting up the simulation domain.

necsim-rust is built in a modular way to reduce code duplication and allow the user (and other programmers) to plug together different components to customise the simulated scenario, the algorithm it is simulated with as well as finer implementation details. Currently, necsim-rust supports four built-in scenarios:
- spatially-explicit simulation
- non-spatial simulation
- spatially-implicit simulation with migration from a non-spatial metacommunity to a non-spatial local community
- (almost) infinite simulation with normal dispersal [Work In Progress]

## Installation

First, you need to clone the necsim-rust GitHub repository:
```shell
> git clone https://github.com/MoritzLangenstein/necsim-rust.git
```
necsim-rust is written in the [Rust language](https://www.rust-lang.org/tools/install), which must be installed in your `PATH` first. As it also uses some nightly features, you must switch to the nightly toolchain to built the library and tool:
```shell
> cd necsim-rust
> rustup override set nightly
```
If you want to use the CUDA-based implementation, you also need to install the following:
```shell
> cargo install ptx-linker -f
> rustup target add nvptx64-nvidia-cuda
```

## Project structure

necsim-rust consists of the following crates:
- rustcoalescence/: `rustcoalescence` provides the command-line interface.
- necsim/:
    - linker/: `necsim-linker` is a custom linker used during the compilation.
    - core/: `necsim-core` declares the core structs, simulation cogs traits, as well as the generic `Simulation`.
    - impls/:
        - no-std/: `necsim-impls-no-std` contains implementations of cogs that **do not** require the Rust standard library
        - std/: `necsim-impls-std` contains implementations of cogs that **do** require the Rust standard library
        - cuda/: `necsim-impls-cuda` contains implementations of CUDA specific cogs
    - algorithms/:
        - classical/: `necsim-classical` instantiates the classical coalescence algorithm (approximation, fast for low speciation probabilities)
        - gillespie/: `necsim-gillespie` instantiates the coalescence simulation based on the Gillespie algorithm (most accurate, slowest)
        - skipping-gillespie/: `necsim-skipping-gillespie` instantiates the coalescence simulation based on the Gillespie algorithm and skips events which do not change the state of the simulation (most accurate, fastest)
        - cuda/: `necsim-cuda` instantiates the coalescence algorithm on a CUDA 3.5 capable GPU as an embarrassingly parallel problem
        - independent/: `necsim-independent` instantiates the coalescence algorithm on the CPU as an embarrassingly parallel problem
- rust-cuda/: `rust-cuda` provides automatically derivable traits to add more (but not full) type safety to sharing data structures between the CPU and GPU
- third-party/: this folder contains multiple third-party crates which had to be modified (mostly to enable `no-std` support)

## Compilation

To compile `rustcoalescence`, you need to decide which algorithms you want to compile with it. You can enable any of the four provided algorithms by enabling its corresponding feature of the same name. For instance, to compile all CPU-based algorithms, you can use
```shell
> cargo rustc --release --manifest-path rustcoalescence/Cargo.toml --features necsim-classical --features necsim-gillespie --features necsim-skipping-gillespie --features necsim-independent
```
To compile with CUDA support, you first need to compile the custom `necsim-linker`:
```shell
> cargo build --release --manifest-path necsim/linker/Cargo.toml
```
Next, you need to ensure that the dynamic CUDA libraries are in the `LD_LIBRARY_PATH`:
```shell
> LIBRARY_PATH="$LD_LIBRARY_PATH" cargo rustc --release --manifest-path rustcoalescence/Cargo.toml --features necsim-cuda -- -Clinker=target/release/necsim-linker
```
In either case, you can then run `rustcoalescence` using:
```
> target/release/rustcoalescence
```

## Running rustcoalescence

`rustcoalescence` accepts command line arguments according to the following format:
```shell
> rustcoalescence --algorithm <algorithm> --sample <sample-percentage> --seed <seed> --speciation <speciation-probability-per-generation> <SUBCOMMAND>
```
Here, the parameters have the following semantics:
- `<algorithm>` is one of `classical`, `gillespie`, `skipping-gillespie`, `cuda` or `independent`, depending on which algorithms it was compiled with to support.
- `<sample-percentage>` refers to the percentage of individuals who should be simulated and must be between `0.0` and `1.0`.
- `<seed>` is the 64bit unsigned seed with which the simulation is initialised.
- `<speciation-probability-per-generation>` refers to the probability with which an individual mutates into a new species at every generation.

`SUBCOMMAND` refers to the supported scenarios, which currently are:
- spatially-explicit: the landscape and dispersal of species are loaded from TIFF files. The parameters shown below have the following semantics:
    - `<habitat-map>` is the path to a TIFF file storing grayscale u32 habitat values with dimensions `W x H`. The maps/ folder contains two habitat maps for testing.
    - `<dispersal-map>` is the path to a TIFF file storing grayscale f64 dispersal weights with dimensions `WxH x WxH`, i.e. the `i`th row of the image stores dispersal from the habitat cell `(i % W, i / W)`. Note that `rustcoalescence` checks that the habitat and dispersal maps make sense in combination. The maps/ folder contains two dispersal maps for testing.
    - `[--strict-load]` is an optional flag to disable GDAL GeoTiff map loading compatibility features. When disabled, `rustcoalescence` will check for and handle GDAL no data values and potential rounding errors in the habitat map.
```shell
> rustcoalescence ... in-memory <habitat-map> <dispersal-map> [--strict-load]
```
- non-spatial: the individuals live uniformly with equal probability to disperse anywhere else. The parameters shown below have the following semantics:
    - `<area>` specifies the non-spatial area that the individuals will inhabit. It can be either one-dimensional `A` or two-dimensional `AxB`.
    - `<deme>` specifies the number of individuals that will be able to cohabit each space in the area. It is functionally equivalent to double the deme or to double the area (though it might impact the performance, and the result of one execution when using the Gillespie algorithm).
    - `[--spatial]` is an optional flag which allows using the spatially explicit simulation cogs to simulate the non-spatial scenario instead of specialised non-spatial cogs. This flag is mostly used to verify both scenarios are implemented correctly.
```shell
> rustcoalescence ... non-spatial <area> <deme> [--spatial]
```
- spatially-implicit: the individuals live uniformly with equal probability to disperse anywhere else. The parameters shown below have the following semantics:
    - `<local-area>` specifies the non-spatial area that the individuals in the local community will inhabit. It can be either one-dimensional `A` or two-dimensional `AxB`. Note that individuals in the local community will not be able to speciate. Also note that the `<sample-percentage>` parameter will only apply to the local community.
    - `<local-deme>` specifies the number of individuals that will be able to cohabit each space in the local community area. It is functionally equivalent to double the deme or to double the area (though it might impact the performance, and the result of one execution when using the Gillespie algorithm).
    - `<meta-area>` specifies the non-spatial area that the individuals in the meta-community will inhabit. It can be either one-dimensional `A` or two-dimensional `AxB`. Note that the `<speciation-probability-per-generation>` parameter only applies to individuals in the meta-community.
    - `<meta-deme>` specifies the number of individuals that will be able to cohabit each space in the meta-community area. It is functionally equivalent to double the deme or to double the area (though it might impact the performance, and the result of one execution when using the Gillespie algorithm).
    - `<migration-probability-per-generation>` refers to the probability with which an individual in the local community migrates to the metacommunity at every generation.
    - `[--dynamic-meta]` is an optional flag to use a dynamic metacommunity instead of a static one. Specifically, this means that different species can have lived at the same location of the metacommunity at different points in time. The spatially-implicit model usually assumes that the metacommunity is static but of infinite size, instead.
```shell
> rustcoalescence ... spatially-implicit --local-area <local-area> --local-deme <local-deme> --meta-area <meta-area> --meta-deme <meta-deme> --migration <migration-probability-per-generation> [--dynamic-meta]
```
- (almost) infinite: all individuals start in a perfect circle and can disperse anywhere in the (almost) infinite landscape (each 32bit coordinate wraps around). The parameters shown below have the following semantics:
    - `<radius>` specifies the radius of the circle from which the individuals will be sampled. Note that the number of individuals, and therefore the runtime, scales quadratically with the radius.
    - `<sigma>` specifies the standard deviation of the normal distribution that will be used as the dispersal kernel.
```shell
> rustcoalescence ... almost-infinite <radius> <sigma>
```

## GDAL GeoTiff compatibility

pycoalescence and necsim both used GDAL to load habitat and dispersal maps. As rustcoalescence is more strict about type checking the TIFF files, you can use the following commands to convert and compress your GeoTIFF files:
```shell
> gdal_translate -ot Uint32 -strict -co "COMPRESS=LZW" input_habitat.tif output_habitat.tif
> gdal_translate -ot Float64 -strict -co "COMPRESS=LZW" input_dispersal.tif output_dispersal.tif
```
