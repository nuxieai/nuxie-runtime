//! Public `Luau::DenseHashMap2` wrapper.

use crate::records::dense_hash_table::{DenseDefault, DenseEq, DenseEqDefault, DenseHasher};
use crate::records::dense_hash_table2::{DenseHashTable2, ItemInterfaceMap2};
use crate::type_aliases::dense_hash_default::DenseHashDefault;

type MapImpl<K, V, H, E> = DenseHashTable2<K, (K, V), ItemInterfaceMap2<K, V>, H, E>;

#[derive(Clone)]
pub struct DenseHashMap2<K, V, H = DenseHashDefault<K>, E = DenseEqDefault<K>> {
    impl_: MapImpl<K, V, H, E>,
}

impl<K, V, H, E> DenseHashMap2<K, V, H, E>
where
    K: Clone + DenseDefault,
    V: DenseDefault,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default,
{
    pub fn new(buckets: usize) -> Self {
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
        &mut self.impl_.data[bucket].1
    }

    pub fn find(&self, key: &K) -> Option<&V> {
        self.impl_
            .find(key)
            .map(|bucket| &self.impl_.data[bucket].1)
    }

    pub fn find_mut(&mut self, key: &K) -> Option<&mut V> {
        self.impl_
            .find(key)
            .map(|bucket| &mut self.impl_.data[bucket].1)
    }

    pub fn contains(&self, key: &K) -> bool {
        self.impl_.find(key).is_some()
    }

    pub fn erase(&mut self, key: &K) {
        self.impl_.erase(key);
    }

    pub fn try_insert(&mut self, key: K, value: V) -> (&mut V, bool) {
        self.impl_.rehash_if_full(&key);
        let before = self.impl_.size();
        let bucket = self.impl_.insert_unsafe(key);
        let fresh = self.impl_.size() > before;

        if fresh {
            self.impl_.data[bucket].1 = value;
        }

        (&mut self.impl_.data[bucket].1, fresh)
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
