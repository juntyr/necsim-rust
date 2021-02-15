// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A `HashMap` wrapper that holds key-value pairs in insertion order.
//!
//! # Examples
//!
//! ```
//! use linked_hash_map::LinkedHashMap;
//!
//! let mut map = LinkedHashMap::new();
//! map.insert(2, 20);
//! map.insert(1, 10);
//! map.insert(3, 30);
//! assert_eq!(map[&1], 10);
//! assert_eq!(map[&2], 20);
//! assert_eq!(map[&3], 30);
//! ```

#![deny(clippy::pedantic)]
#![forbid(missing_docs)]

use std::{
    borrow::Borrow,
    collections::hash_map::{self, HashMap},
    hash::{BuildHasher, Hash, Hasher},
    mem,
    ops::{Index, IndexMut},
};

use slab::Slab;

struct KeyRef<K, V> {
    slab: *const Slab<Node<K, V>>,
    index: usize,
}

impl<K, V> KeyRef<K, V> {
    #[allow(clippy::borrowed_box)]
    fn new(slab: &Box<Slab<Node<K, V>>>, index: usize) -> Self {
        Self {
            slab: &**slab,
            index,
        }
    }
}

struct Node<K, V> {
    next: usize,
    prev: usize,
    key: K,
    value: V,
}

/// A linked hash map.
pub struct LinkedHashMap<K, V, S = hash_map::RandomState> {
    map: HashMap<KeyRef<K, V>, usize, S>,
    head: Option<usize>,
    slab: Box<Slab<Node<K, V>>>,
}

impl<K, V, S> Drop for LinkedHashMap<K, V, S> {
    fn drop(&mut self) {
        if let Some(head) = self.head.take() {
            mem::forget(self.slab.remove(head))
        }
    }
}

impl<K: Hash, V> Hash for KeyRef<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let slab: &Slab<Node<K, V>> = unsafe { &*self.slab };

        slab[self.index].key.hash(state)
    }
}

impl<K: PartialEq, V> PartialEq for KeyRef<K, V> {
    fn eq(&self, other: &Self) -> bool {
        let slab: &Slab<Node<K, V>> = unsafe { &*self.slab };

        slab[self.index].key.eq(&slab[other.index].key)
    }
}

impl<K: Eq, V> Eq for KeyRef<K, V> {}

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

impl<K, V, Q: ?Sized> Borrow<Qey<Q>> for KeyRef<K, V>
where
    K: Borrow<Q>,
{
    fn borrow(&self) -> &Qey<Q> {
        let slab: &Slab<Node<K, V>> = unsafe { &*self.slab };

        Qey::from_ref(slab[self.index].key.borrow())
    }
}

impl<K, V> Node<K, V> {
    fn new(k: K, v: V) -> Self {
        Node {
            key: k,
            value: v,
            next: 0,
            prev: 0,
        }
    }
}

impl<K: Hash + Eq, V> LinkedHashMap<K, V> {
    /// Creates a linked hash map.
    #[must_use]
    pub fn new() -> Self {
        Self::with_map(HashMap::new())
    }

    /// Creates an empty linked hash map with the given initial capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_map(HashMap::with_capacity(capacity))
    }
}

impl<K, V, S> LinkedHashMap<K, V, S> {
    #[inline]
    fn detach(&mut self, node_index: usize) {
        let node = &mut self.slab[node_index];

        let node_next = node.next;
        let node_prev = node.prev;

        self.slab[node_prev].next = node_next;
        self.slab[node_next].prev = node_prev;
    }

    #[inline]
    fn attach(&mut self, node_index: usize) {
        if let Some(head) = self.head {
            let head_next = self.slab[head].next;

            let node = &mut self.slab[node_index];

            node.next = head_next;
            node.prev = head;

            self.slab[head].next = node_index;
            self.slab[head_next].prev = node_index;
        }
    }

    fn ensure_guard_node(&mut self) {
        if self.head.is_none() {
            // allocate the guard node if not present
            unsafe {
                let node = std::mem::MaybeUninit::uninit().assume_init();

                let head_index = self.slab.insert(node);

                self.head = Some(head_index);
                let head = &mut self.slab[head_index];

                head.next = head_index;
                head.prev = head_index;
            }
        }
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> LinkedHashMap<K, V, S> {
    fn with_map(map: HashMap<KeyRef<K, V>, usize, S>) -> Self {
        let slab = Box::new(Slab::with_capacity(map.capacity()));
        
        LinkedHashMap {
            map,
            head: None,
            slab,
        }
    }

    /// Creates an empty linked hash map with the given initial hash builder.
    pub fn with_hasher(hash_builder: S) -> Self {
        Self::with_map(HashMap::with_hasher(hash_builder))
    }

    /// Creates an empty linked hash map with the given initial capacity and
    /// hash builder.
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self::with_map(HashMap::with_capacity_and_hasher(capacity, hash_builder))
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// into the map. The map may reserve more space to avoid frequent
    /// allocations.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows `usize.`
    pub fn reserve(&mut self, additional: usize) {
        self.map.reserve(additional);
        self.slab.reserve(additional);
    }

    /// Shrinks the capacity of the map as much as possible. It will drop down
    /// as much as possible while maintaining the internal rules and
    /// possibly leaving some space in accordance with the resize policy.
    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit();
        self.slab.shrink_to_fit();
    }

    /// Inserts a key-value pair into the map. If the key already existed, the
    /// old value is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_hash_map::LinkedHashMap;
    /// let mut map = LinkedHashMap::new();
    ///
    /// map.insert(1, "a");
    /// map.insert(2, "b");
    /// assert_eq!(map[&1], "a");
    /// assert_eq!(map[&2], "b");
    /// ```
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.ensure_guard_node();

        #[allow(clippy::option_if_let_else)]
        if let Some(node_index) = self.map.get(Qey::from_ref(&k)) {
            let node_index = *node_index;
            let node = &mut self.slab[node_index];

            let old_val = mem::replace(&mut node.value, v);

            // Existing node, just update LRU position
            self.detach(node_index);
            self.attach(node_index);

            Some(old_val)
        } else {
            let node_index = self.slab.insert(Node::new(k, v));

            self.map
                .insert(KeyRef::new(&self.slab, node_index), node_index);
            self.attach(node_index);

            None
        }
    }

    /// Checks if the map contains the given key.
    pub fn contains_key<Q: ?Sized>(&self, k: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.map.contains_key(Qey::from_ref(k))
    }

    /// Returns the value corresponding to the key in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_hash_map::LinkedHashMap;
    /// let mut map = LinkedHashMap::new();
    ///
    /// map.insert(1, "a");
    /// map.insert(2, "b");
    /// map.insert(2, "c");
    /// map.insert(3, "d");
    ///
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.get(&2), Some(&"c"));
    /// ```
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        match self.map.get(Qey::from_ref(k)) {
            Some(node_index) => Some(&self.slab[*node_index].value),
            None => None,
        }
    }

    /// Returns the mutable reference corresponding to the key in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_hash_map::LinkedHashMap;
    /// let mut map = LinkedHashMap::new();
    ///
    /// map.insert(1, "a");
    /// map.insert(2, "b");
    ///
    /// *map.get_mut(&1).unwrap() = "c";
    /// assert_eq!(map.get(&1), Some(&"c"));
    /// ```
    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        match self.map.get(Qey::from_ref(k)) {
            Some(node_index) => Some(&mut self.slab[*node_index].value),
            None => None,
        }
    }

    /// Returns the value corresponding to the key in the map.
    ///
    /// If value is found, it is moved to the end of the list.
    /// This operation can be used in implemenation of LRU cache.
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_hash_map::LinkedHashMap;
    /// let mut map = LinkedHashMap::new();
    ///
    /// map.insert(1, "a");
    /// map.insert(2, "b");
    /// map.insert(3, "d");
    ///
    /// assert_eq!(map.get_refresh(&2), Some(&mut "b"));
    /// ```
    pub fn get_refresh<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        match self.map.get(Qey::from_ref(k)) {
            Some(node_index) => {
                let node_index = *node_index;

                self.detach(node_index);
                self.attach(node_index);

                Some(&mut self.slab[node_index].value)
            },
            None => None,
        }
    }

    /// Removes and returns the value corresponding to the key from the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_hash_map::LinkedHashMap;
    /// let mut map = LinkedHashMap::new();
    ///
    /// map.insert(2, "a");
    ///
    /// assert_eq!(map.remove(&1), None);
    /// assert_eq!(map.remove(&2), Some("a"));
    /// assert_eq!(map.remove(&2), None);
    /// assert_eq!(map.len(), 0);
    /// ```
    pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        let removed = self.map.remove(Qey::from_ref(k));
        removed.map(|node_index| {
            self.detach(node_index);

            self.slab.remove(node_index).value
        })
    }

    /// Returns the maximum number of key-value pairs the map can hold without
    /// reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_hash_map::LinkedHashMap;
    /// let mut map: LinkedHashMap<i32, &str> = LinkedHashMap::new();
    /// let capacity = map.capacity();
    /// ```
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    /// Removes the first entry.
    ///
    /// Can be used in implementation of LRU cache.
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_hash_map::LinkedHashMap;
    /// let mut map = LinkedHashMap::new();
    /// map.insert(1, 10);
    /// map.insert(2, 20);
    /// map.pop_front();
    /// assert_eq!(map.get(&1), None);
    /// assert_eq!(map.get(&2), Some(&20));
    /// ```
    #[inline]
    pub fn pop_front(&mut self) -> Option<(K, V)> {
        if self.is_empty() {
            return None;
        }

        let head = self.head?;
        let lru = self.slab[head].prev;

        self.detach(lru);

        self.map
            .remove(Qey::from_ref(&self.slab[lru].key))
            .map(|lru| {
                let node = self.slab.remove(lru);

                (node.key, node.value)
            })
    }

    /// Gets the first entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_hash_map::LinkedHashMap;
    /// let mut map = LinkedHashMap::new();
    /// map.insert(1, 10);
    /// map.insert(2, 20);
    /// assert_eq!(map.front(), Some((&1, &10)));
    /// ```
    #[inline]
    pub fn front(&self) -> Option<(&K, &V)> {
        if self.is_empty() {
            return None;
        }

        let head = self.head?;
        let lru = &self.slab[self.slab[head].prev];

        Some((&lru.key, &lru.value))
    }

    /// Removes the last entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_hash_map::LinkedHashMap;
    /// let mut map = LinkedHashMap::new();
    /// map.insert(1, 10);
    /// map.insert(2, 20);
    /// map.pop_back();
    /// assert_eq!(map.get(&1), Some(&10));
    /// assert_eq!(map.get(&2), None);
    /// ```
    #[inline]
    pub fn pop_back(&mut self) -> Option<(K, V)> {
        if self.is_empty() {
            return None;
        }

        let head = self.head?;
        let mru = self.slab[head].next;

        self.detach(mru);

        self.map
            .remove(Qey::from_ref(&self.slab[mru].key))
            .map(|lru| {
                let node = self.slab.remove(lru);

                (node.key, node.value)
            })
    }

    /// Gets the last entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use linked_hash_map::LinkedHashMap;
    /// let mut map = LinkedHashMap::new();
    /// map.insert(1, 10);
    /// map.insert(2, 20);
    /// assert_eq!(map.back(), Some((&2, &20)));
    /// ```
    #[inline]
    pub fn back(&mut self) -> Option<(&K, &V)> {
        if self.is_empty() {
            return None;
        }

        let head = self.head?;
        let mru = &self.slab[self.slab[head].next];

        Some((&mru.key, &mru.value))
    }

    /// Returns the number of key-value pairs in the map.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns whether the map is currently empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the map's hasher.
    pub fn hasher(&self) -> &S {
        self.map.hasher()
    }

    /// Clears the map of all key-value pairs.
    pub fn clear(&mut self) {
        self.map.clear();

        if let Some(head) = self.head.take() {
            mem::forget(self.slab.remove(head))
        }

        self.slab.clear();
    }
}

impl<'a, K, V, S, Q: ?Sized> Index<&'a Q> for LinkedHashMap<K, V, S>
where
    K: Hash + Eq + Borrow<Q>,
    S: BuildHasher,
    Q: Eq + Hash,
{
    type Output = V;

    fn index(&self, index: &'a Q) -> &V {
        self.get(index).expect("no entry found for key")
    }
}

impl<'a, K, V, S, Q: ?Sized> IndexMut<&'a Q> for LinkedHashMap<K, V, S>
where
    K: Hash + Eq + Borrow<Q>,
    S: BuildHasher,
    Q: Eq + Hash,
{
    fn index_mut(&mut self, index: &'a Q) -> &mut V {
        self.get_mut(index).expect("no entry found for key")
    }
}

impl<K: Hash + Eq, V, S: BuildHasher + Default> Default for LinkedHashMap<K, V, S> {
    fn default() -> Self {
        Self::with_hasher(S::default())
    }
}

unsafe impl<K: Send, V: Send, S: Send> Send for LinkedHashMap<K, V, S> {}

unsafe impl<K: Sync, V: Sync, S: Sync> Sync for LinkedHashMap<K, V, S> {}
