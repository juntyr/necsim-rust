use std::{
    marker::PhantomData,
    mem::{offset_of, MaybeUninit},
    os::raw::{c_int, c_void},
};

use mpi::{
    collective::UnsafeUserOperation,
    datatype::{Equivalence, UserDatatype},
    ffi::MPI_Datatype,
    topology::SimpleCommunicator,
    traits::CommunicatorCollectives,
};

use necsim_core::lineage::MigratingLineage;
use necsim_core_bond::PositiveF64;

#[repr(C)]
#[derive(Clone, Copy, mpi::traits::Equivalence)]
struct TimeRank {
    time: f64,
    rank: u32,
}

pub fn reduce_lexicographic_min_time_rank(
    world: &SimpleCommunicator,
    time: PositiveF64,
    rank: u32,
) -> (PositiveF64, u32) {
    let local_time_rank = TimeRank {
        time: time.get(),
        rank,
    };
    let mut global_min_time_rank = local_time_rank;

    let operation =
        unsafe { UnsafeUserOperation::commutative(unsafe_reduce_lexicographic_min_time_rank_op) };

    world.all_reduce_into(&local_time_rank, &mut global_min_time_rank, &operation);

    // Safety: min time comes from reduction of all PositiveF64
    let min_time = unsafe { PositiveF64::new_unchecked(global_min_time_rank.time) };
    let min_rank = global_min_time_rank.rank;

    (min_time, min_rank)
}

#[cfg(not(all(msmpi, target_arch = "x86")))]
unsafe extern "C" fn unsafe_reduce_lexicographic_min_time_rank_op(
    invec: *mut c_void,
    inoutvec: *mut c_void,
    len: *mut c_int,
    datatype: *mut MPI_Datatype,
) {
    unsafe_reduce_lexicographic_min_time_rank_op_inner(invec, inoutvec, len, datatype);
}

#[cfg(all(msmpi, target_arch = "x86"))]
unsafe extern "stdcall" fn unsafe_reduce_lexicographic_min_time_rank_op(
    invec: *mut c_void,
    inoutvec: *mut c_void,
    len: *mut c_int,
    datatype: *mut MPI_Datatype,
) {
    unsafe_reduce_lexicographic_min_time_rank_op_inner(invec, inoutvec, len, datatype);
}

#[inline]
unsafe fn unsafe_reduce_lexicographic_min_time_rank_op_inner(
    invec: *mut c_void,
    inoutvec: *mut c_void,
    len: *mut c_int,
    datatype: *mut MPI_Datatype,
) {
    debug_assert!(*len == 1);
    debug_assert!(*datatype == mpi::raw::AsRaw::as_raw(&TimeRank::equivalent_datatype()));

    reduce_lexicographic_min_time_rank_inner(&*invec.cast(), &mut *inoutvec.cast());
}

#[inline]
fn reduce_lexicographic_min_time_rank_inner(local: &TimeRank, accumulator: &mut TimeRank) {
    if (local.time < accumulator.time)
        || (local.time <= accumulator.time && local.rank < accumulator.rank)
    {
        *accumulator = *local;
    }
}

#[repr(transparent)]
pub struct MpiMigratingLineage(MigratingLineage);

impl MpiMigratingLineage {
    pub fn from_slice(slice: &[MigratingLineage]) -> &[MpiMigratingLineage] {
        // Safety: cast to transparent newtype wrapper
        unsafe {
            std::slice::from_raw_parts(slice.as_ptr().cast::<MpiMigratingLineage>(), slice.len())
        }
    }

    pub fn from_mut_uninit_slice(
        slice: &mut [MaybeUninit<MigratingLineage>],
    ) -> &mut [MaybeUninit<MpiMigratingLineage>] {
        // Safety: cast to transparent newtype wrapper
        unsafe {
            std::slice::from_raw_parts_mut(
                slice
                    .as_mut_ptr()
                    .cast::<MaybeUninit<MpiMigratingLineage>>(),
                slice.len(),
            )
        }
    }
}

unsafe impl Equivalence for MpiMigratingLineage {
    type Out = UserDatatype;

    #[allow(clippy::cast_possible_wrap)]
    fn equivalent_datatype() -> Self::Out {
        // Ensure compilation breaks if a new field is added
        let MigratingLineage {
            global_reference: _,
            prior_time: _,
            event_time: _,
            coalescence_rng_sample: _,
            dispersal_target: _,
            dispersal_origin: _,
            tie_breaker: _,
        };

        UserDatatype::structured(
            &[1, 1, 1, 1, 2, 3, 1],
            &[
                offset_of!(MigratingLineage, global_reference) as mpi::Address,
                offset_of!(MigratingLineage, prior_time) as mpi::Address,
                offset_of!(MigratingLineage, event_time) as mpi::Address,
                offset_of!(MigratingLineage, coalescence_rng_sample) as mpi::Address,
                offset_of!(MigratingLineage, dispersal_target) as mpi::Address,
                offset_of!(MigratingLineage, dispersal_origin) as mpi::Address,
                offset_of!(MigratingLineage, tie_breaker) as mpi::Address,
            ],
            &[
                u64::equivalent_datatype(),
                f64::equivalent_datatype(),
                f64::equivalent_datatype(),
                f64::equivalent_datatype(),
                u32::equivalent_datatype(),
                u32::equivalent_datatype(),
                i8::equivalent_datatype(),
            ],
        )
    }
}

pub fn reduce_partitioning_data<
    T: serde::Serialize + serde::de::DeserializeOwned,
    F: 'static + Copy + Fn(T, T) -> T,
>(
    world: &SimpleCommunicator,
    data: T,
    fold: F,
) -> T {
    let local_ser = postcard::to_stdvec(&data).expect("MPI data failed to serialize");
    let mut global_ser = Vec::with_capacity(local_ser.len());

    let operation =
        unsafe { UnsafeUserOperation::commutative(unsafe_reduce_partitioning_data_op::<T, F>) };

    world.all_reduce_into(local_ser.as_slice(), &mut global_ser, &operation);

    postcard::from_bytes(&global_ser).expect("MPI data failed to deserialize")
}

#[cfg(not(all(msmpi, target_arch = "x86")))]
unsafe extern "C" fn unsafe_reduce_partitioning_data_op<
    T: serde::Serialize + serde::de::DeserializeOwned,
    F: 'static + Copy + Fn(T, T) -> T,
>(
    invec: *mut c_void,
    inoutvec: *mut c_void,
    len: *mut c_int,
    datatype: *mut MPI_Datatype,
) {
    unsafe_reduce_partitioning_data_op_inner::<T, F>(invec, inoutvec, len, datatype);
}

#[cfg(all(msmpi, target_arch = "x86"))]
unsafe extern "stdcall" fn unsafe_reduce_partitioning_data_op<
    T: serde::Serialize + serde::de::DeserializeOwned,
    F: 'static + Copy + Fn(T, T) -> T,
>(
    invec: *mut c_void,
    inoutvec: *mut c_void,
    len: *mut c_int,
    datatype: *mut MPI_Datatype,
) {
    unsafe_reduce_partitioning_data_op_inner::<T, F>(invec, inoutvec, len, datatype);
}

#[inline]
unsafe fn unsafe_reduce_partitioning_data_op_inner<
    T: serde::Serialize + serde::de::DeserializeOwned,
    F: 'static + Copy + Fn(T, T) -> T,
>(
    invec: *mut c_void,
    inoutvec: *mut c_void,
    len: *mut c_int,
    datatype: *mut MPI_Datatype,
) {
    debug_assert!(*len == 1);
    debug_assert!(*datatype == mpi::raw::AsRaw::as_raw(&TimeRank::equivalent_datatype()));

    reduce_partitioning_data_op_inner::<T, F>(&*invec.cast(), &mut *inoutvec.cast());
}

#[inline]
fn reduce_partitioning_data_op_inner<
    T: serde::Serialize + serde::de::DeserializeOwned,
    F: 'static + Copy + Fn(T, T) -> T,
>(
    local_ser: &[u8],
    global_ser: &mut Vec<u8>,
) {
    union Magic<T, F: 'static + Copy + Fn(T, T) -> T> {
        func: F,
        unit: (),
        marker: PhantomData<T>,
    }

    let local_de: T = postcard::from_bytes(local_ser).expect("MPI data failed to deserialize");
    let global_de: T = postcard::from_bytes(global_ser).expect("MPI data failed to deserialize");

    const { assert!(std::mem::size_of::<F>() == 0) };
    const { assert!(std::mem::align_of::<F>() == 1) };
    let func: F = unsafe { Magic { unit: () }.func };

    let folded = func(local_de, global_de);

    global_ser.clear();

    postcard::to_io(&folded, global_ser).expect("MPI data failed to serialize");
}
