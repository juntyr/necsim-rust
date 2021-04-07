use core::marker::PhantomData;

use crate::reporter::boolean::{Boolean, False, True};

#[allow(clippy::module_name_repetitions)]
pub struct MaybeUsed<'a, T, B: Boolean> {
    inner: &'a T,
    _used: PhantomData<B>,
}

pub type Used<'a, T> = MaybeUsed<'a, T, True>;
pub type Unused<'a, T> = MaybeUsed<'a, T, False>;

impl<'a, T> From<&'a T> for Unused<'a, T> {
    fn from(inner: &'a T) -> Self {
        Self::new(inner)
    }
}

impl<'a, T, B: Boolean> MaybeUsed<'a, T, B> {
    #[must_use]
    pub fn use_in<F: FnOnce(&T)>(self, func: F) -> Used<'a, T> {
        func(&self.inner);

        self.into()
    }

    #[must_use]
    pub fn ignore(self) -> Self {
        self.into()
    }

    #[must_use]
    pub fn maybe_use_in<O: Boolean, F: FnOnce(&T)>(self, func: F) -> MaybeUsed<'a, T, O> {
        if O::VALUE {
            func(&self.inner);
        }

        self.into()
    }
}

impl<'a, T, B: Boolean> MaybeUsed<'a, T, B> {
    pub fn new(inner: &'a T) -> Self {
        MaybeUsed {
            inner,
            _used: PhantomData,
        }
    }

    pub(super) fn unused(&self) -> Unused<'a, T> {
        Unused::new(self.inner)
    }

    pub(super) fn into<O: Boolean>(self) -> MaybeUsed<'a, T, O> {
        MaybeUsed::new(self.inner)
    }
}
