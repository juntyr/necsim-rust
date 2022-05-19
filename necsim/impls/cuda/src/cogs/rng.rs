use core::marker::PhantomData;

use const_type_layout::TypeGraphLayout;
use rust_cuda::safety::StackOnly;

use necsim_core::cogs::{MathsCore, Rng};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, rust_cuda::common::LendRustToCuda)]
#[cuda(free = "M", free = "R")]
pub struct CudaRng<M: MathsCore, R>
where
    R: Rng<M> + StackOnly + ~const TypeGraphLayout,
{
    inner: R,
    marker: PhantomData<M>,
}

impl<M: MathsCore, R: Rng<M> + StackOnly + ~const TypeGraphLayout> Clone for CudaRng<M, R> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            marker: PhantomData::<M>,
        }
    }
}

impl<M: MathsCore, R: Rng<M> + StackOnly + ~const TypeGraphLayout> From<R> for CudaRng<M, R> {
    #[must_use]
    #[inline]
    fn from(rng: R) -> Self {
        Self {
            inner: rng,
            marker: PhantomData::<M>,
        }
    }
}

impl<M: MathsCore, R: Rng<M> + StackOnly + ~const TypeGraphLayout> CudaRng<M, R> {
    pub fn into(self) -> R {
        self.inner
    }
}

impl<M: MathsCore, R: Rng<M> + StackOnly + ~const TypeGraphLayout> Rng<M> for CudaRng<M, R> {
    type Generator = R::Generator;
    type Sampler = R::Sampler;

    fn generator(&mut self) -> &mut Self::Generator {
        self.inner.generator()
    }

    fn map_generator<F: FnOnce(Self::Generator) -> Self::Generator>(self, map: F) -> Self {
        let CudaRng { inner, marker } = self;

        CudaRng {
            inner: inner.map_generator(map),
            marker,
        }
    }

    fn with<F: FnOnce(&mut Self::Generator, &Self::Sampler) -> Q, Q>(&mut self, inner: F) -> Q {
        self.inner.with(inner)
    }
}
