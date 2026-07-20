//! Faithful port of `Luau::DenseHashSet` — a fast `unordered_set` alternative
//! that does not support erasing. Reference:
//! `luau/Common/include/Luau/DenseHash.h:471-556`.
//!
//! Construction takes an `empty_key` sentinel that must never be inserted as a
//! real key (it marks free slots).

use crate::records::const_iterator::ConstIterator;
use crate::records::dense_hash_table::{
    DenseEq, DenseEqDefault, DenseHashTable, DenseHasher, ItemInterfaceSet,
};
use crate::type_aliases::dense_hash_default::DenseHashDefault;

type SetImpl<K, H, E> = DenseHashTable<K, K, ItemInterfaceSet<K>, H, E>;

pub struct DenseHashSet<K, H = DenseHashDefault<K>, E = DenseEqDefault<K>> {
    pub(crate) impl_: SetImpl<K, H, E>,
}

impl<K: Clone, H: Clone, E: Clone> Clone for DenseHashSet<K, H, E> {
    fn clone(&self) -> Self {
        DenseHashSet {
            impl_: self.impl_.clone(),
        }
    }
}

impl<K: core::fmt::Debug, H, E> core::fmt::Debug for DenseHashSet<K, H, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DenseHashSet")
            .field("impl_", &self.impl_)
            .finish()
    }
}

impl<K, H, E> DenseHashSet<K, H, E>
where
    K: Clone,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default,
{
    /// `DenseHashSet(empty_key, buckets = 0)`. Reference: `DenseHash.h:482-485`.
    pub fn new(empty_key: K) -> Self {
        DenseHashSet {
            impl_: DenseHashTable::new(empty_key, 0),
        }
    }

    /// `insert` — inserts `key` if absent and returns a reference to the stored
    /// key. Reference: `DenseHash.h:492-496`.
    pub fn insert(&mut self, key: K) -> &K {
        self.impl_.rehash_if_full(&key);
        let idx = self.impl_.insert_unsafe(key);
        &self.impl_.data[idx]
    }

    /// `const Key* find(const Key&) const`. Reference: `DenseHash.h:498-501`.
    pub fn find(&self, key: &K) -> Option<&K> {
        self.impl_.find(key).map(|idx| &self.impl_.data[idx])
    }

    /// std-style alias for generated Rust that spelled C++ `find` as `get`.
    pub fn get(&self, key: &K) -> Option<&K> {
        self.find(key)
    }

    /// Mutable analogue of `insert`: inserts `key` if absent and returns a
    /// mutable reference to the stored key. C++ `insert` returns `const Key&`
    /// and callers reach for `const_cast<Key&>` when they need to fix up a
    /// stored entry in place (e.g. `AstNameTable::getOrAddWithType` rewriting a
    /// non-owned name pointer to an allocator-owned copy of the same bytes —
    /// the hash/eq are unchanged so the slot stays valid). This exposes that
    /// mutation soundly. Not a separate C++ method; the mutable access is the
    /// faithful Rust spelling of the `const_cast` idiom.
    pub fn insert_mut(&mut self, key: K) -> &mut K {
        self.impl_.rehash_if_full(&key);
        let idx = self.impl_.insert_unsafe(key);
        &mut self.impl_.data[idx]
    }

    /// Mutable analogue of `find`.
    pub fn find_mut(&mut self, key: &K) -> Option<&mut K> {
        match self.impl_.find(key) {
            Some(idx) => Some(&mut self.impl_.data[idx]),
            None => None,
        }
    }

    /// `contains`. Reference: `DenseHash.h:503-506`.
    pub fn contains(&self, key: &K) -> bool {
        self.impl_.find(key).is_some()
    }

    /// `size`. Reference: `DenseHash.h:508-511`.
    pub fn size(&self) -> usize {
        self.impl_.size()
    }

    /// `empty`. Reference: `DenseHash.h:513-516`.
    pub fn empty(&self) -> bool {
        self.impl_.size() == 0
    }

    /// `clear`. Reference: `DenseHash.h:487-490`.
    pub fn clear(&mut self) {
        self.impl_.clear();
    }

    /// `begin()/end()` iteration, yielding `&Key`.
    pub fn iter(&self) -> ConstIterator<'_, K, K, ItemInterfaceSet<K>, H, E> {
        ConstIterator {
            table: &self.impl_,
            index: self.impl_.first_occupied(),
        }
    }
}

/// `operator==` / `operator!=` — set equality. Reference: `DenseHash.h:538-555`.
impl<K, H, E> PartialEq for DenseHashSet<K, H, E>
where
    K: Clone,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default,
{
    fn eq(&self, other: &Self) -> bool {
        if self.size() != other.size() {
            return false;
        }
        for k in self.iter() {
            if !other.contains(k) {
                return false;
            }
        }
        true
    }
}

/// C++ members declare their empty key inline (`DenseHashSet<T> x{emptyT}`);
/// Rust `#[derive(Default)]` on containing structs needs this. The empty key
/// comes from the element's DenseDefault sentinel.
impl<K, H, E> Default for DenseHashSet<K, H, E>
where
    K: Clone + crate::records::dense_hash_table::DenseDefault,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default,
{
    fn default() -> Self {
        Self::new(K::dense_default())
    }
}
