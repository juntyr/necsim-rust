use serde::Deserialize;

#[derive(Deserialize)]
#[serde(remote = "necsim_skipping_gillespie::ParallelismMode")]
enum ParallelismModeDef {
    Optimistic,
    Lockstep,
    OptimisticLockstep,
}

#[derive(Deserialize)]
#[serde(remote = "necsim_skipping_gillespie::SkippingGillespieArguments")]
#[serde(default = "necsim_skipping_gillespie::SkippingGillespieArguments::default")]
pub struct ArgumentsDef {
    #[serde(with = "ParallelismModeDef")]
    pub parallelism_mode: necsim_skipping_gillespie::ParallelismMode,
}
