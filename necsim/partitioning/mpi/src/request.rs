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

    pub fn test_if_request<S: for<'t> FnOnce(Option<&'t mut T>) -> Q, Q>(
        &mut self,
        do_test: S,
    ) -> Q {
        match self.request.take().map(Request::test) {
            Some(Ok(_)) => do_test(Some(self.value)),
            Some(Err(request)) => {
                self.request = Some(request);
                do_test(None)
            },
            None => do_test(None),
        }
    }

    #[must_use]
    pub fn request_if_data_then_test<
        R: for<'r> FnOnce(&'r mut T, &'r LocalScope<'r>) -> Request<'r, &'r LocalScope<'r>>,
        S: for<'t> FnOnce(Option<&'t mut T>) -> Q,
        Q,
    >(
        &mut self,
        do_request: R,
        do_test: S,
    ) -> Q {
        let request = self.request.take().unwrap_or_else(|| {
            let request = do_request(self.value, reduce_scope(self.scope));

            // Safety: upgrade of request scope back to the scope 'a
            unsafe { std::mem::transmute(request) }
        });

        match request.test() {
            Ok(_) => do_test(Some(self.value)),
            Err(request) => {
                self.request = Some(request);
                do_test(None)
            },
        }
    }

    #[allow(clippy::let_unit_value)]
    pub fn request_if_data<
        R: for<'r> FnOnce(&'r mut T, &'r LocalScope<'r>) -> Request<'r, &'r LocalScope<'r>>,
    >(
        &mut self,
        do_request: R,
    ) {
        let _: () = self.request_if_data_then_test(do_request, |_| ());
    }

    #[must_use]
    pub fn check<C: for<'c> FnOnce(Option<&'c T>) -> Q, Q>(&self, do_check: C) -> Q {
        match &self.request {
            None => do_check(Some(self.value)),
            Some(_) => do_check(None),
        }
    }

    #[must_use]
    pub fn is_request(&self) -> bool {
        self.request.is_some()
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
