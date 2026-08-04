use core::mem;

use crate::records::dense_hash_map2::DenseHashMap2;
use crate::records::dense_hash_table::{DenseDefault, DenseEq, DenseHasher};
use crate::records::dense_hash_table2::DenseHashTable2;

impl<K, V, H, E> DenseHashMap2<K, V, H, E>
where
    K: Clone + DenseDefault,
    V: DenseDefault,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default,
{
    pub fn new() -> Self {
        Self::with_buckets(0)
    }

    pub fn with_buckets(buckets: usize) -> Self {
        Self {
            impl_: DenseHashTable2::new(buckets),
        }
    }

    pub fn clear(&mut self) {
        self.impl_.clear(32);
    }

    pub fn clear_with_threshold(&mut self, threshold_to_destroy: usize) {
        self.impl_.clear(threshold_to_destroy);
    }

    pub fn get_or_insert(&mut self, key: K) -> &mut V {
        self.impl_.rehash_if_full(&key);
        let bucket = self.impl_.insert_unsafe(key);
        &mut self.impl_.data[bucket]
            .as_mut()
            .expect("occupied bucket")
            .1
    }

    pub fn find(&self, key: &K) -> Option<&V> {
        self.impl_
            .find(key)
            .map(|bucket| &self.impl_.data[bucket].as_ref().expect("occupied bucket").1)
    }

    pub fn find_mut(&mut self, key: &K) -> Option<&mut V> {
        self.impl_
            .find(key)
            .map(|bucket| {
                &mut self.impl_.data[bucket]
                    .as_mut()
                    .expect("occupied bucket")
                    .1
            })
    }

    pub fn contains(&self, key: &K) -> bool {
        self.impl_.find(key).is_some()
    }

    pub fn erase(&mut self, key: &K) {
        self.impl_.erase(key);
    }

    /// C++ `try_insert(const Value&)`: copy only when the key is fresh.
    pub fn try_insert(&mut self, key: K, value: &V) -> (&mut V, bool)
    where
        V: Clone,
    {
        self.impl_.rehash_if_full(&key);
        let before = self.impl_.size();
        let bucket = self.impl_.insert_unsafe(key);
        let fresh = self.impl_.size() > before;

        if fresh {
            self.impl_.data[bucket]
                .as_mut()
                .expect("occupied bucket")
                .1 = value.clone();
        }

        (
            &mut self.impl_.data[bucket]
                .as_mut()
                .expect("occupied bucket")
                .1,
            fresh,
        )
    }

    /// C++ `try_insert(Value&&)`: move only when the key is fresh.
    ///
    /// The source remains untouched when the key already exists, matching the
    /// conditional `std::move(value)` in the C++ overload.
    pub fn try_insert_move(&mut self, key: K, value: &mut V) -> (&mut V, bool) {
        self.impl_.rehash_if_full(&key);
        let before = self.impl_.size();
        let bucket = self.impl_.insert_unsafe(key);
        let fresh = self.impl_.size() > before;

        if fresh {
            self.impl_.data[bucket]
                .as_mut()
                .expect("occupied bucket")
                .1 = mem::replace(value, V::dense_default());
        }

        (
            &mut self.impl_.data[bucket]
                .as_mut()
                .expect("occupied bucket")
                .1,
            fresh,
        )
    }

    pub fn size(&self) -> usize {
        self.impl_.size()
    }

    pub fn empty(&self) -> bool {
        self.impl_.size() == 0
    }

    pub fn is_empty(&self) -> bool {
        self.empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.impl_.iter().map(|item| (&item.0, &item.1))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&K, &mut V)> {
        self.impl_.iter_mut().map(|item| (&item.0, &mut item.1))
    }
}

impl<K, V, H, E> Default for DenseHashMap2<K, V, H, E>
where
    K: Clone + DenseDefault,
    V: DenseDefault,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default,
{
    fn default() -> Self {
        Self::new()
    }
}
