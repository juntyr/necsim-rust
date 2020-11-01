pub mod cuda {
    #[cfg(not(feature = "cuda"))]
    #[allow(clippy::module_name_repetitions)]
    pub trait RustToCuda {}

    #[cfg(feature = "cuda")]
    #[allow(clippy::module_name_repetitions)]
    pub trait RustToCuda: necsim_cuda::common::RustToCuda {}
}
