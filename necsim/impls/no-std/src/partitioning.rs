use core::num::NonZeroU32;

use necsim_core::reporter::Reporter;

use crate::reporter::{GuardedReporter, ReporterContext};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Partitioning: Sized {
    type LocalPartition<P: ReporterContext>: LocalPartition<P>;

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

    fn into_local_partition<P: ReporterContext>(
        self,
        reporter_context: P,
    ) -> Self::LocalPartition<P>;
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait LocalPartition<P: ReporterContext>: Sized {
    type Reporter: Reporter;

    fn get_reporter(&mut self) -> &mut Self::Reporter;

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

    fn wait_for_termination(&mut self) -> bool;
}

pub struct MonolithicLocalPartition<P: ReporterContext> {
    reporter: GuardedReporter<P::Reporter, P::Finaliser>,
}

#[contract_trait]
impl<P: ReporterContext> LocalPartition<P> for MonolithicLocalPartition<P> {
    type Reporter = P::Reporter;

    fn get_reporter(&mut self) -> &mut Self::Reporter {
        &mut self.reporter
    }

    fn is_root(&self) -> bool {
        true
    }

    fn get_partition_rank(&self) -> u32 {
        0
    }

    fn get_number_of_partitions(&self) -> NonZeroU32 {
        unsafe { NonZeroU32::new_unchecked(1) }
    }

    fn reduce_global_time_steps(&self, local_time: f64, local_steps: u64) -> (f64, u64) {
        (local_time, local_steps)
    }

    fn wait_for_termination(&mut self) -> bool {
        false
    }
}

impl<P: ReporterContext> MonolithicLocalPartition<P> {
    pub fn from_reporter(reporter_guard: GuardedReporter<P::Reporter, P::Finaliser>) -> Self {
        Self {
            reporter: reporter_guard,
        }
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
    type LocalPartition<P: ReporterContext> = MonolithicLocalPartition<P>;

    fn is_monolithic(&self) -> bool {
        true
    }

    fn is_root(&self) -> bool {
        true
    }

    fn get_number_of_partitions(&self) -> NonZeroU32 {
        unsafe { NonZeroU32::new_unchecked(1) }
    }

    fn into_local_partition<P: ReporterContext>(
        self,
        reporter_context: P,
    ) -> Self::LocalPartition<P> {
        MonolithicLocalPartition::from_reporter(reporter_context.build_guarded())
    }
}
