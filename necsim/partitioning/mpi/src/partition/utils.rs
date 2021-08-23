use std::os::raw::{c_int, c_void};

use mpi::{
    collective::UnsafeUserOperation, datatype::Equivalence, ffi::MPI_Datatype,
    topology::SystemCommunicator, traits::CommunicatorCollectives,
};

use necsim_core_bond::PositiveF64;

#[derive(mpi::traits::Equivalence, PartialEq, Copy, Clone)]
#[repr(C)]
pub struct TimePartition {
    pub time: PositiveF64,
    pub partition: u32,
}

pub fn reduce_lexicographic_min_time_partition(
    world: SystemCommunicator,
    time_partition: TimePartition,
) -> TimePartition {
    let local_time_partition = time_partition;
    let mut global_min_time_partition = local_time_partition;

    let operation = unsafe {
        UnsafeUserOperation::commutative(unsafe_reduce_lexicographic_min_time_partition_op)
    };

    world.all_reduce_into(
        &local_time_partition,
        &mut global_min_time_partition,
        &operation,
    );

    global_min_time_partition
}

#[cfg(not(all(msmpi, target_arch = "x86")))]
unsafe extern "C" fn unsafe_reduce_lexicographic_min_time_partition_op(
    invec: *mut c_void,
    inoutvec: *mut c_void,
    len: *mut c_int,
    datatype: *mut MPI_Datatype,
) {
    unsafe_reduce_lexicographic_min_time_partition_op_inner(invec, inoutvec, len, datatype);
}

#[cfg(all(msmpi, target_arch = "x86"))]
unsafe extern "stdcall" fn unsafe_reduce_lexicographic_min_time_partition_op(
    invec: *mut c_void,
    inoutvec: *mut c_void,
    len: *mut c_int,
    datatype: *mut MPI_Datatype,
) {
    unsafe_reduce_lexicographic_min_time_partition_op_inner(invec, inoutvec, len, datatype);
}

#[inline]
unsafe fn unsafe_reduce_lexicographic_min_time_partition_op_inner(
    invec: *mut c_void,
    inoutvec: *mut c_void,
    len: *mut c_int,
    datatype: *mut MPI_Datatype,
) {
    debug_assert!(*len == 1);
    debug_assert!(*datatype == mpi::raw::AsRaw::as_raw(&TimePartition::equivalent_datatype()));

    reduce_lexicographic_min_time_partition_inner(&*invec.cast(), &mut *inoutvec.cast());
}

#[inline]
fn reduce_lexicographic_min_time_partition_inner(
    local: &TimePartition,
    accumulator: &mut TimePartition,
) {
    if (local.time < accumulator.time)
        || (local.time == accumulator.time && local.partition < accumulator.partition)
    {
        *accumulator = *local;
    }
}
