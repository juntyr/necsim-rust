use core::ops::Deref;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Backup: Sized {
    #[must_use]
    unsafe fn backup_unchecked(&self) -> Self;

    fn backup(&self) -> BackedUp<Self> {
        BackedUp(unsafe { self.backup_unchecked() })
    }
}

pub struct BackedUp<T: Backup>(pub(crate) T);

impl<T: Backup> Deref for BackedUp<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
