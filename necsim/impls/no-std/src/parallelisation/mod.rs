pub mod independent;
pub mod monolithic;

pub enum Status {
    Paused,
    Done,
}

impl Status {
    #[must_use]
    pub fn paused(paused: bool) -> Self {
        if paused {
            Self::Paused
        } else {
            Self::Done
        }
    }
}
