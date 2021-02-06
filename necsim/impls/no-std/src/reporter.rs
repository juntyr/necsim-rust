use core::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use necsim_core::reporter::Reporter;

#[allow(clippy::module_name_repetitions)]
pub trait ReporterContext {
    type Reporter: Reporter;
    type Finaliser: FnOnce(Self::Reporter);

    fn build_guarded(self) -> GuardedReporter<Self::Reporter, Self::Finaliser>;
}

#[allow(clippy::module_name_repetitions)]
pub struct GuardedReporter<R: Reporter, F: FnOnce(R)> {
    reporter: ManuallyDrop<R>,
    finaliser: ManuallyDrop<F>,
}

impl<R: Reporter, F: FnOnce(R)> GuardedReporter<R, F> {
    pub fn from(reporter: R, finaliser: F) -> Self {
        Self {
            reporter: ManuallyDrop::new(reporter),
            finaliser: ManuallyDrop::new(finaliser),
        }
    }
}

impl<R: Reporter, F: FnOnce(R)> Deref for GuardedReporter<R, F> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.reporter
    }
}

impl<R: Reporter, F: FnOnce(R)> DerefMut for GuardedReporter<R, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reporter
    }
}

impl<R: Reporter, F: FnOnce(R)> Drop for GuardedReporter<R, F> {
    fn drop(&mut self) {
        // Safety:
        //  - the destructor is only called once
        //  - self.reporter and self.finaliser will be dropped in the call
        unsafe {
            ManuallyDrop::take(&mut self.finaliser)
                .call_once((ManuallyDrop::take(&mut self.reporter),))
        }
    }
}
