use serde::Deserialize;

#[derive(Deserialize)]
#[serde(remote = "necsim_cuda::DedupMode")]
enum DedupModeDef {
    Static(usize),
    Dynamic(f64),
    None,
}

#[derive(Deserialize)]
#[serde(remote = "necsim_cuda::CudaArguments")]
#[serde(default = "necsim_cuda::CudaArguments::default")]
pub struct ArgumentsDef {
    pub ptx_jit: bool,
    pub delta_t: f64,
    pub block_size: u32,
    pub grid_size: u32,
    pub step_slice: usize,
    #[serde(with = "DedupModeDef")]
    pub dedup_mode: necsim_cuda::DedupMode,
}
