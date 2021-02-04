use core::num::NonZeroU32;

use necsim_core::reporter::Reporter;

use crate::reporter::ReporterContext;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Partitioning: Sized {
    type ParallelPartition<R: Reporter>: ParallelPartition<R, Partitioning = Self>;

    fn is_monolithic(&self) -> bool;

    #[debug_ensures(
        self.is_monolithic() -> ret,
        "monolithic partition is always root"
    )]
    fn is_root(&self) -> bool;

    #[debug_ensures(
        self.is_monolithic() == (ret.get() == 1),
        "there is only one monolithic partition"
    )]
    fn get_number_of_partitions(&self) -> NonZeroU32;

    fn with_local_partition<
        P: ReporterContext,
        Q,
        F: for<'r> FnOnce(
            Result<
                &mut MonolithicPartition<'r, P::Reporter>,
                &mut Self::ParallelPartition<P::Reporter>,
            >,
        ) -> Q,
    >(
        &mut self,
        reporter_context: P,
        inner: F,
    ) -> Q;
}

pub trait Partition<R: Reporter> {
    type Reporter: Reporter;

    fn get_reporter(&mut self) -> &mut Self::Reporter;
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait ParallelPartition<R: Reporter>: Partition<R> {
    // Should be Partitioning<ParallelPartition<R> = Self>
    //  after https://github.com/rust-lang/rust/pull/79554
    type Partitioning: Partitioning;

    fn is_root(&self) -> bool;

    #[debug_ensures(
        ret < self.get_number_of_partitions().get(),
        "partition rank is in range [0, self.get_number_of_partitions())"
    )]
    fn get_partition_rank(&self) -> u32;

    #[debug_ensures(
        ret.get() > 1,
        "there are more than one parallel partitions"
    )]
    fn get_number_of_partitions(&self) -> NonZeroU32;

    fn reduce_global_time_steps(&self, local_time: f64, local_steps: u64) -> (f64, u64);
}

pub struct MonolithicPartition<'r, R: Reporter> {
    reporter: &'r mut R,
}

impl<'r, R: Reporter> Partition<R> for MonolithicPartition<'r, R> {
    type Reporter = R;

    fn get_reporter(&mut self) -> &mut Self::Reporter {
        self.reporter
    }
}

impl<'r, R: Reporter> MonolithicPartition<'r, R> {
    pub fn from_reporter(reporter: &'r mut R) -> Self {
        Self { reporter }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct MonolithicPartitioning(());

impl Default for MonolithicPartitioning {
    fn default() -> Self {
        Self(())
    }
}

#[contract_trait]
impl Partitioning for MonolithicPartitioning {
    type ParallelPartition<R: Reporter> = !;

    fn is_monolithic(&self) -> bool {
        true
    }

    fn is_root(&self) -> bool {
        true
    }

    fn get_number_of_partitions(&self) -> NonZeroU32 {
        unsafe { NonZeroU32::new_unchecked(1) }
    }

    fn with_local_partition<
        P: ReporterContext,
        Q,
        F: for<'r> FnOnce(
            Result<
                &mut MonolithicPartition<'r, P::Reporter>,
                &mut Self::ParallelPartition<P::Reporter>,
            >,
        ) -> Q,
    >(
        &mut self,
        reporter_context: P,
        inner: F,
    ) -> Q {
        reporter_context
            .with_reporter(|reporter| inner(Ok(&mut MonolithicPartition::from_reporter(reporter))))
    }
}

#[contract_trait]
impl<R: Reporter> ParallelPartition<R> for ! {
    type Partitioning = MonolithicPartitioning;

    fn is_root(&self) -> bool {
        unreachable!("! cannot be constructed")
    }

    fn get_partition_rank(&self) -> u32 {
        unreachable!("! cannot be constructed")
    }

    fn get_number_of_partitions(&self) -> NonZeroU32 {
        unreachable!("! cannot be constructed")
    }

    fn reduce_global_time_steps(&self, _local_time: f64, _local_steps: u64) -> (f64, u64) {
        unreachable!("! cannot be constructed")
    }
}

impl<R: Reporter> Partition<R> for ! {
    type Reporter = R;

    fn get_reporter(&mut self) -> &mut Self::Reporter {
        unreachable!("! cannot be constructed")
    }
}
