use core::num::NonZeroU32;

use necsim_core::reporter::Reporter;

use crate::reporter::ReporterContext;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Partitioning<R: Reporter>: Sized {
    type ParallelPartition: ParallelPartition<R, Partitioning = Self>;

    fn is_monolithic(&self) -> bool;

    #[debug_ensures(
        self.is_monolithic() == (ret.get() == 1),
        "there is only one monolithic partition"
    )]
    fn get_number_of_partitions(&self) -> NonZeroU32;

    fn with_local_partition<
        P: ReporterContext<Reporter = R>,
        Q,
        F: for<'r> FnOnce(
            Result<&mut MonolithicPartition<'r, P::Reporter>, &mut Self::ParallelPartition>,
        ) -> Q,
    >(
        &mut self,
        reporter_context: P,
        inner: F,
    ) -> Q;
}

pub trait Partition<R: Reporter> {
    fn get_reporter(&mut self) -> &mut R;
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait ParallelPartition<R: Reporter>: Partition<R> {
    type Partitioning: Partitioning<R, ParallelPartition = Self>;

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
}

pub struct MonolithicPartition<'r, R: Reporter> {
    reporter: &'r mut R,
}

impl<'r, R: Reporter> Partition<R> for MonolithicPartition<'r, R> {
    fn get_reporter(&mut self) -> &mut R {
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
impl<R: Reporter> Partitioning<R> for MonolithicPartitioning {
    type ParallelPartition = !;

    fn is_monolithic(&self) -> bool {
        true
    }

    fn get_number_of_partitions(&self) -> NonZeroU32 {
        unsafe { NonZeroU32::new_unchecked(1) }
    }

    fn with_local_partition<
        P: ReporterContext<Reporter = R>,
        Q,
        F: for<'r> FnOnce(
            Result<&mut MonolithicPartition<'r, P::Reporter>, &mut Self::ParallelPartition>,
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

    fn get_partition_rank(&self) -> u32 {
        unreachable!("! cannot be constructed")
    }

    fn get_number_of_partitions(&self) -> NonZeroU32 {
        unreachable!("! cannot be constructed")
    }
}

impl<R: Reporter> Partition<R> for ! {
    fn get_reporter(&mut self) -> &mut R {
        unreachable!("! cannot be constructed")
    }
}
