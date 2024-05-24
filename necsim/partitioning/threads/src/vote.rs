use std::sync::{Arc, Barrier, RwLock};

#[derive(Clone)]
pub struct Vote<T: Clone> {
    shared: Arc<SharedVote<T>>,
    generation: Generation,
}

struct SharedVote<T: Clone> {
    data: RwLock<GenerationalData<T>>,
    barrier: Barrier,
}

impl<T: Clone> Vote<T> {
    #[must_use]
    pub fn new(n: usize) -> Self
    where
        T: Default,
    {
        Self::new_with_dummy(n, T::default())
    }

    #[must_use]
    pub fn new_with_dummy(n: usize, dummy: T) -> Self {
        Self {
            shared: Arc::new(SharedVote {
                data: RwLock::new(GenerationalData {
                    data: dummy,
                    generation: Generation::first(),
                }),
                barrier: Barrier::new(n),
            }),
            generation: Generation::first().next(),
        }
    }

    pub fn vote(&mut self, vote: impl FnOnce(Option<&T>) -> T) -> T {
        {
            let mut generational_data = self.shared.data.write().unwrap();

            if generational_data.generation == self.generation {
                generational_data.data = vote(Some(&generational_data.data));
            } else {
                generational_data.data = vote(None);
                generational_data.generation = self.generation;
            }
        }

        self.shared.barrier.wait();

        let result = {
            let data = self.shared.data.read().unwrap();
            data.data.clone()
        };

        self.generation = self.generation.next();

        self.shared.barrier.wait();

        result
    }
}

struct GenerationalData<T> {
    data: T,
    generation: Generation,
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct Generation(bool);

impl Generation {
    #[must_use]
    pub fn first() -> Self {
        Self(false)
    }

    #[must_use]
    pub fn next(self) -> Self {
        Self(!self.0)
    }
}
