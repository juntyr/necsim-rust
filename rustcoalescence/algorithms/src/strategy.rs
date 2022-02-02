use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct RestartFixUpStrategy {
    #[serde(alias = "deme", alias = "ood")]
    pub out_of_deme: OutOfDemeStrategy,
    #[serde(alias = "habitat", alias = "ooh")]
    pub out_of_habitat: OutOfHabitatStrategy,
    #[serde(alias = "dup", alias = "coa")]
    pub coalescence: CoalescenceStrategy,
}

impl Default for RestartFixUpStrategy {
    fn default() -> Self {
        Self {
            out_of_deme: OutOfDemeStrategy::Abort,
            out_of_habitat: OutOfHabitatStrategy::Abort,
            coalescence: CoalescenceStrategy::Abort,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Serialize, Deserialize)]
pub enum OutOfDemeStrategy {
    Abort,
    Dispersal,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Serialize, Deserialize)]
pub enum OutOfHabitatStrategy {
    Abort,
    #[serde(alias = "Uniform")]
    UniformDispersal,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Serialize, Deserialize)]
pub enum CoalescenceStrategy {
    Abort,
    Coalescence,
}
