use thiserror::Error;

#[allow(clippy::module_name_repetitions)]
#[derive(Error, Debug)]
pub enum InMemoryDispersalSamplerError {
    #[error("The size of the dispersal map was inconsistent with the size of the habitat map.")]
    InconsistentDispersalMapSize,
    #[error(
        "{}{}{}",
        "Habitat cells must disperse somewhere AND ",
        "non-habitat cells must not disperse AND ",
        "dispersal must only target habitat cells."
    )]
    InconsistentDispersalProbabilities,
}
