use ptx_builder::{builder::Builder, error::Result, reporter::CargoAdapter};

fn main() -> Result<()> {
    let builder = Builder::new("kernel")?;

    CargoAdapter::with_env_var("CUDA_PTX_KERNEL").build(builder);
}
