use serde::Deserialize;

#[derive(Deserialize)]
#[serde(remote = "necsim_independent::DedupMode")]
enum DedupModeDef {
    Static(usize),
    Dynamic(f64),
    None,
}

#[derive(Deserialize)]
#[serde(remote = "necsim_independent::PartitionMode")]
pub enum PartitionModeDef {
    Landscape,
    Individuals,
    Probabilistic,
}

#[derive(Deserialize)]
#[serde(remote = "necsim_independent::IndependentArguments")]
#[serde(default = "necsim_independent::IndependentArguments::default")]
pub struct ArgumentsDef {
    pub delta_t: f64,
    pub step_slice: usize,
    #[serde(with = "DedupModeDef")]
    pub dedup_mode: necsim_independent::DedupMode,
    #[serde(with = "PartitionModeDef")]
    pub partition_mode: necsim_independent::PartitionMode,
}
