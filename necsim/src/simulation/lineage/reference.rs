#[derive(Copy, Clone)]
pub struct LineageReference(pub(super) usize);

impl LineageReference {
    #[must_use]
    pub(super) fn new(reference: usize) -> Self {
        Self(reference)
    }
}
