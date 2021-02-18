use std::num::NonZeroUsize;

use lru_set::LruSet;

#[test]
fn test_insert() {
    let mut set = LruSet::with_capacity(2);
    println!("{:?}", set);
    assert_eq!(set.len(), 0);

    assert!(set.insert(1));
    println!("{:?}", set);
    assert_eq!(set.len(), 1);

    assert!(set.insert(2));
    println!("{:?}", set);
    assert_eq!(set.len(), 2);

    assert!(!set.insert(1));
    println!("{:?}", set);
    assert_eq!(set.len(), 2);

    assert!(set.insert(3));
    println!("{:?}", set);
    assert_eq!(set.len(), 2);

    assert!(!set.insert(1));
    println!("{:?}", set);
    assert_eq!(set.len(), 2);

    assert!(set.insert(2));
    println!("{:?}", set);
    assert_eq!(set.len(), 2);
}

#[test]
fn test_send_sync() {
    fn is_send_sync<T: Send + Sync>() {}

    is_send_sync::<LruSet<u32>>();
}
