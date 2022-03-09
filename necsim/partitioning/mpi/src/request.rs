use mpi::request::{CancelGuard, LocalScope, Request};

#[allow(clippy::module_name_repetitions)]
pub struct DataOrRequest<'a, T: ?Sized> {
    value: &'a mut T,
    scope: &'a LocalScope<'a>,
    request: Option<Request<'a, &'a LocalScope<'a>>>,
}

impl<'a, T: ?Sized> DataOrRequest<'a, T> {
    #[must_use]
    pub fn new(value: &'a mut T, scope: &'a LocalScope<'a>) -> Self {
        Self {
            value,
            scope,
            request: None,
        }
    }

    #[must_use]
    pub fn get_data(&self) -> Option<&T> {
        match &self.request {
            None => Some(self.value),
            Some(_) => None,
        }
    }

    #[must_use]
    pub fn test_for_data_mut(&mut self) -> Option<&mut T> {
        match self.request.take().map(Request::test) {
            None | Some(Ok(_)) => Some(self.value),
            Some(Err(request)) => {
                self.request = Some(request);

                None
            },
        }
    }

    pub fn request_if_data<
        R: for<'r> FnOnce(&'r mut T, &'r LocalScope<'r>) -> Request<'r, &'r LocalScope<'r>>,
    >(
        &mut self,
        do_request: R,
    ) {
        if self.request.is_none() {
            let request = do_request(self.value, reduce_scope(self.scope));

            // Safety: upgrade of request scope back to the scope 'a
            self.request = Some(unsafe { std::mem::transmute(request) });
        }
    }
}

impl<'a, T: ?Sized> Drop for DataOrRequest<'a, T> {
    fn drop(&mut self) {
        if let Some(request) = self.request.take() {
            std::mem::drop(CancelGuard::from(request));
        }
    }
}

pub fn reduce_scope<'s, 'p: 's>(scope: &'s LocalScope<'p>) -> &'s LocalScope<'s> {
    // Safety: 'p outlives 's, so reducing the scope from 'p to 's is safe
    unsafe { std::mem::transmute(scope) }
}
