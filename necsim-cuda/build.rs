use ptx_builder::{error::Result, prelude::*};

fn main() -> Result<()> {
    CargoAdapter::with_env_var("KERNEL_PTX_PATH").build(Builder::new("kernel")?);
}
