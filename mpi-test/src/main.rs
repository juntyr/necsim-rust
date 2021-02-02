use mpi::{request::WaitGuard, traits::*};

use necsim_core::{
    cogs::CoalescenceRngSample,
    landscape::{IndexedLocation, Location},
    lineage::GlobalLineageReference,
};

#[derive(Equivalence)]
struct MigratingLineage {
    global_reference: GlobalLineageReference,
    dispersal_origin: IndexedLocation,
    dispersal_target: Location,
    event_time: f64,
    coalescence_rng_sample: CoalescenceRngSample,
}

fn main() {
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let size = world.size();
    let rank = world.rank();

    println!("{:?}/{:?}", rank + 1, size);

    // IDEA:
    // 1) initialise universe in mpi impl component
    // 2) if compiled without mpi support, always return monlithic config
    // 3) combination of habitats and samplers (TODO) are able to take config
    // and split
    //    - every habitat must be able to split into disjoint subparts
    //    - optionally add splitting sampler on top (quasi-random sampling)
    //    - must be able to check that no splitting sampling took place
    //      statically for non-independent sims
    // 4) base case samplers will be habitat specific
    // 5) samplers can be combined (maybe base on iterators)
    // 6) sample percentage will be a sampler (though it might need variants if
    // an exact output N is requested) 7) this also opens up an opportunity
    // for sample maps / sample functions

    // let next_rank = if rank + 1 < size { rank + 1 } else { 0 };
    // let previous_rank = if rank > 0 { rank - 1 } else { size - 1 };
    //
    // let msg = vec![rank, 2 * rank, 4 * rank];
    // mpi::request::scope(|scope| {
    // let _sreq = WaitGuard::from(
    // world
    // .process_at_rank(next_rank)
    // .immediate_send(scope, &msg[..]),
    // );
    //
    // let (msg, status) = world.any_process().receive_vec();
    //
    // println!(
    // "Process {} got message {:?}.\nStatus is: {:?}",
    // rank, msg, status
    // );
    // let x = status.source_rank();
    // assert_eq!(x, previous_rank);
    // assert_eq!(vec![x, 2 * x, 4 * x], msg);
    //
    // let root_rank = 0;
    // let root_process = world.process_at_rank(root_rank);
    //
    // let mut a;
    // if world.rank() == root_rank {
    // a = vec![2, 4, 8, 16];
    // println!("Root broadcasting value: {:?}.", &a[..]);
    // } else {
    // a = vec![0; 4];
    // }
    // root_process.broadcast_into(&mut a[..]);
    // println!("Rank {} received value: {:?}.", world.rank(), &a[..]);
    // assert_eq!(&a[..], &[2, 4, 8, 16]);
    // });
}
