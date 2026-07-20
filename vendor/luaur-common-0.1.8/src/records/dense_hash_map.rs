//! Faithful port of `Luau::DenseHashMap` — a fast `unordered_map` alternative
//! that does not support erasing and uses `find()` instead of returning an
//! iterator. Reference: `luau/Common/include/Luau/DenseHash.h:558-669`.
//!
//! Construction takes an `empty_key` sentinel that must never be used as a real
//! key; this is part of the observable interface (downstream code constructs
//! `DenseHashMap<K, V>(emptyKey)` at hundreds of sites).

use core::marker::PhantomData;

use crate::records::const_iterator::ConstIterator;
use crate::records::dense_hash_table::{
    DenseEq, DenseEqDefault, DenseHashTable, DenseHasher, ItemInterfaceMap,
};
use crate::records::iterator::MutIterator;
use crate::type_aliases::dense_hash_default::DenseHashDefault;

type MapImpl<K, V, H, E> = DenseHashTable<K, (K, V), ItemInterfaceMap<K, V>, H, E>;

pub struct DenseHashMap<K, V, H = DenseHashDefault<K>, E = DenseEqDefault<K>> {
    pub(crate) impl_: MapImpl<K, V, H, E>,
}

impl<K: Clone, V: Clone, H: Clone, E: Clone> Clone for DenseHashMap<K, V, H, E> {
    fn clone(&self) -> Self {
        DenseHashMap {
            impl_: self.impl_.clone(),
        }
    }
}

impl<K: core::fmt::Debug, V: core::fmt::Debug, H, E> core::fmt::Debug for DenseHashMap<K, V, H, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DenseHashMap")
            .field("impl_", &self.impl_)
            .finish()
    }
}

impl<K, V, H, E> DenseHashMap<K, V, H, E>
where
    K: Clone,
    V: crate::records::dense_hash_table::DenseDefault,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default + Clone,
{
    /// `DenseHashMap(empty_key, buckets = 0)`. Reference: `DenseHash.h:570-573`.
    pub fn new(empty_key: K) -> Self {
        DenseHashMap {
            impl_: DenseHashTable::new(empty_key, 0),
        }
    }

    /// `operator[]` — inserts a default value when absent and returns a mutable
    /// reference to the slot's value. Reference: `DenseHash.h:580-585`.
    pub fn get_or_insert(&mut self, key: K) -> &mut V {
        self.impl_.rehash_if_full(&key);
        let idx = self.impl_.insert_unsafe(key);
        &mut self.impl_.data[idx].1
    }

    /// `const Value* find(const Key&) const`. Reference: `DenseHash.h:588-593`.
    pub fn find(&self, key: &K) -> Option<&V> {
        self.impl_.find(key).map(|idx| &self.impl_.data[idx].1)
    }

    /// std-style alias for generated Rust that spelled C++ `find` as `get`.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.find(key)
    }

    /// `Value* find(const Key&)`. Reference: `DenseHash.h:596-601`.
    pub fn find_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.impl_.find(key) {
            Some(idx) => Some(&mut self.impl_.data[idx].1),
            None => None,
        }
    }

    /// std-style alias for generated Rust that spelled C++ `find` as `get_mut`.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.find_mut(key)
    }

    /// `contains`. Reference: `DenseHash.h:603-606`.
    /// std-style alias for `contains` — translations use HashMap idioms.
    pub fn contains_key(&self, key: &K) -> bool {
        self.contains(key)
    }

    pub fn contains(&self, key: &K) -> bool {
        self.impl_.find(key).is_some()
    }

    /// `try_insert` — returns `(value_ref, fresh)`, where `fresh` is true only
    /// when the key was newly inserted; an existing slot keeps its value.
    /// Reference: `DenseHash.h:608-638`.
    pub fn try_insert(&mut self, key: K, value: V) -> (&mut V, bool) {
        self.impl_.rehash_if_full(&key);

        let before = self.impl_.size();
        let idx = self.impl_.insert_unsafe(key);

        // Value is fresh if container count has increased
        let fresh = self.impl_.size() > before;

        if fresh {
            self.impl_.data[idx].1 = value;
        }

        (&mut self.impl_.data[idx].1, fresh)
    }

    /// `size`. Reference: `DenseHash.h:640-643`.
    pub fn size(&self) -> usize {
        self.impl_.size()
    }

    /// `empty`. Reference: `DenseHash.h:645-648`.
    pub fn empty(&self) -> bool {
        self.impl_.size() == 0
    }

    /// std-style alias for translated code that calls C++ `empty()` as `is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.empty()
    }

    /// `clear`. Reference: `DenseHash.h:575-578`.
    pub fn clear(&mut self) {
        self.impl_.clear();
    }

    /// `begin()/end()` const iteration, yielding `(&Key, &Value)`.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        ConstIterator {
            table: &self.impl_,
            index: self.impl_.first_occupied(),
        }
        .map(|item| (&item.0, &item.1))
    }

    /// `begin()/end()` mutable iteration, yielding `(&Key, &mut Value)`.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        let empty_key = self.impl_.empty_key.clone();
        let eq = self.impl_.eq.clone();
        MutIterator::<K, (K, V), ItemInterfaceMap<K, V>, E> {
            inner: self.impl_.data.iter_mut(),
            empty_key,
            eq,
            _iface: PhantomData,
        }
        .map(|item| (&item.0, &mut item.1))
    }
}
