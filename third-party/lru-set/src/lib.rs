// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A LRU set that holds a maximum number of values in insertion order.

#![deny(clippy::pedantic)]
#![forbid(missing_docs)]

use std::{
    borrow::Borrow,
    collections::hash_map::{self, HashMap},
    hash::{BuildHasher, Hash, Hasher},
    ptr::NonNull,
};

#[derive(Debug)]
struct KeyRef<K> {
    key: NonNull<K>,
}

impl<K> KeyRef<K> {
    fn new(node: NonNull<Node<K>>) -> Self {
        let key = match unsafe { node.as_ref() }.key.as_ref() {
            Some(key) => NonNull::from(key),
            None => unsafe { std::hint::unreachable_unchecked() },
        };

        Self { key }
    }
}

#[derive(Debug)]
struct Node<K> {
    next: NonNull<Node<K>>,
    prev: NonNull<Node<K>>,
    key: Option<K>,
}

/// A least recently used set.
#[derive(Debug)]
pub struct LruSet<K, S = hash_map::RandomState> {
    capacity: usize,
    map: HashMap<KeyRef<K>, NonNull<Node<K>>, S>,
    head: NonNull<Node<K>>,
    free: Option<NonNull<Node<K>>>,
}

impl<K, S> Drop for LruSet<K, S> {
    fn drop(&mut self) {
        let slice: &mut [Node<K>] =
            unsafe { std::slice::from_raw_parts_mut(self.head.as_ptr(), self.capacity + 1) };
        let r#box: Box<[Node<K>]> = unsafe { Box::from_raw(slice) };

        std::mem::drop(r#box)
    }
}

impl<K: Hash> Hash for KeyRef<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { self.key.as_ref() }.hash(state)
    }
}

impl<K: PartialEq> PartialEq for KeyRef<K> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.key.as_ref() }.eq(unsafe { other.key.as_ref() })
    }
}

impl<K: Eq> Eq for KeyRef<K> {}

// This type exists only to support borrowing `KeyRef`s, which cannot be
// borrowed to `Q` directly due to conflicting implementations of `Borrow`. The
// layout of `&Qey<Q>` must be identical to `&Q` in order to support transmuting
// in the `Qey::from_ref` method.
#[derive(Hash, PartialEq, Eq)]
#[repr(transparent)]
struct Qey<Q: ?Sized>(Q);

impl<Q: ?Sized> Qey<Q> {
    fn from_ref(q: &Q) -> &Self {
        unsafe { &*(q as *const Q as *const Qey<Q>) }
    }
}

impl<K, Q: ?Sized> Borrow<Qey<Q>> for KeyRef<K>
where
    K: Borrow<Q>,
{
    fn borrow(&self) -> &Qey<Q> {
        Qey::from_ref(unsafe { self.key.as_ref() }.borrow())
    }
}

impl<K: Hash + Eq> LruSet<K> {
    /// Creates an empty lru set with the given maximum capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_map(capacity, HashMap::with_capacity(capacity))
    }
}

impl<K, S> LruSet<K, S> {
    #[inline]
    fn detach(mut node: NonNull<Node<K>>) {
        unsafe {
            node.as_mut().prev.as_mut().next = node.as_ref().next;
            node.as_mut().next.as_mut().prev = node.as_ref().prev;
        }
    }

    #[inline]
    fn attach(mut head: NonNull<Node<K>>, mut node: NonNull<Node<K>>) {
        unsafe {
            node.as_mut().next = head.as_ref().next;
            node.as_mut().prev = head;

            head.as_mut().next = node;
            node.as_mut().next.as_mut().prev = node;
        }
    }
}

impl<K: Hash + Eq, S: BuildHasher> LruSet<K, S> {
    fn with_map(capacity: usize, map: HashMap<KeyRef<K>, NonNull<Node<K>>, S>) -> Self {
        let mut slab = Vec::with_capacity(capacity + 1);

        let head = unsafe { NonNull::new_unchecked(slab.get_unchecked_mut(0)) };
        slab.push(Node {
            key: None,
            next: head,
            prev: head,
        });

        let free = if capacity > 0 {
            let free = unsafe { NonNull::new_unchecked(slab.get_unchecked_mut(1)) };

            slab.resize_with(capacity + 1, || Node {
                key: None,
                next: free,
                prev: free,
            });

            for i in 1..=capacity {
                Self::attach(free, unsafe {
                    NonNull::new_unchecked(slab.get_unchecked_mut(i))
                })
            }

            Some(free)
        } else {
            None
        };

        Box::into_raw(slab.into_boxed_slice());

        LruSet {
            capacity,
            map,
            head,
            free,
        }
    }

    /// Creates an empty lru set with the given maximum capacity and hash
    /// builder.
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self::with_map(
            capacity,
            HashMap::with_capacity_and_hasher(capacity, hash_builder),
        )
    }

    /// Inserts a value into the map. If the value already existed, false is
    /// returned.
    pub fn insert(&mut self, k: K) -> bool {
        if self.capacity == 0 {
            return true;
        }

        #[allow(clippy::option_if_let_else)]
        if let Some(node) = self.map.get(Qey::from_ref(&k)) {
            // Existing node, just update LRU position
            Self::detach(*node);
            Self::attach(self.head, *node);

            false
        } else if let Some(mut free) = self.free.take() {
            // New node, pull from free list
            Self::detach(free);

            // Check if the free list is still non-empty
            let next_free = unsafe { free.as_ref() }.next;
            if next_free != free {
                self.free = Some(next_free);
            }

            // Insert the key into the node
            unsafe { free.as_mut() }.key = Some(k);

            // Insert and attach the node
            self.map.insert(KeyRef::new(free), free);
            Self::attach(self.head, free);

            true
        } else {
            // New node, reuse LRU
            let mut lru = unsafe { self.head.as_ref() }.prev;
            Self::detach(lru);

            // Remove the prior map entry and drop the prior key
            self.map.remove(&KeyRef::new(lru));
            std::mem::drop(unsafe { lru.as_mut() }.key.replace(k));

            // Insert and attach the node
            self.map.insert(KeyRef::new(lru), lru);
            Self::attach(self.head, lru);

            true
        }
    }

    /// Checks if the map contains the given value.
    pub fn contains<Q: ?Sized>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.map.contains_key(Qey::from_ref(k))
    }

    /// Checks if the map contains the given value.
    ///
    /// If value is found, it is moved to the end of the list.
    /// This operation can be used in implemenation of LRU cache.
    pub fn contains_refresh<Q: ?Sized>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        match self.map.get(Qey::from_ref(k)) {
            Some(node) => {
                // Existing node, just update LRU position
                Self::detach(*node);
                Self::attach(self.head, *node);

                true
            },
            None => false,
        }
    }

    /// Returns the maximum number of values the map can hold.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the number of key-value pairs in the map.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns whether the map is currently empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

unsafe impl<K: Send, S: Send> Send for LruSet<K, S> {}

unsafe impl<K: Sync, S: Sync> Sync for LruSet<K, S> {}
