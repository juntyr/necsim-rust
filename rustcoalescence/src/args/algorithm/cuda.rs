use serde::Deserialize;

use necsim_cuda::CudaArguments as AuxiliaryArguments;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct CudaArguments {
    #[serde(default = "default_ptx_jit")]
    ptx_jit: bool,
    #[serde(default = "default_delta_t")]
    delta_t: f64,
    #[serde(default = "default_block_size")]
    block_size: u32,
    #[serde(default = "default_grid_size")]
    grid_size: u32,
    #[serde(default = "default_step_slice")]
    step_slice: usize,
}

impl Default for CudaArguments {
    fn default() -> Self {
        Self {
            ptx_jit: default_ptx_jit(),
            delta_t: default_delta_t(),
            block_size: default_block_size(),
            grid_size: default_grid_size(),
            step_slice: default_step_slice(),
        }
    }
}

impl Into<AuxiliaryArguments> for CudaArguments {
    fn into(self) -> AuxiliaryArguments {
        AuxiliaryArguments {
            ptx_jit: self.ptx_jit,
            delta_t: self.delta_t,
            block_size: self.block_size,
            grid_size: self.grid_size,
            step_slice: self.step_slice,
        }
    }
}

// Waiting for https://github.com/serde-rs/serde/pull/1490
fn default_ptx_jit() -> bool {
    false
}
fn default_delta_t() -> f64 {
    1.0_f64
}
fn default_block_size() -> u32 {
    32_u32
}
fn default_grid_size() -> u32 {
    256_u32
}
fn default_step_slice() -> usize {
    200_usize
}
