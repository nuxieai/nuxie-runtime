//! Public `Luau::DenseHashSet2` wrapper.

use crate::records::dense_hash_table::{DenseDefault, DenseEq, DenseEqDefault, DenseHasher};
use crate::records::dense_hash_table2::{
    DenseHashTable2, DenseHashTable2Iter, DenseHashTable2IterMut, ItemInterfaceSet2,
};
use crate::type_aliases::dense_hash_default::DenseHashDefault;

type SetImpl<K, H, E> = DenseHashTable2<K, K, ItemInterfaceSet2<K>, H, E>;

#[derive(Clone)]
pub struct DenseHashSet2<K, H = DenseHashDefault<K>, E = DenseEqDefault<K>> {
    impl_: SetImpl<K, H, E>,
}

impl<K, H, E> DenseHashSet2<K, H, E>
where
    K: Clone + DenseDefault,
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

    pub fn insert(&mut self, key: K) -> &K {
        self.impl_.rehash_if_full(&key);
        let bucket = self.impl_.insert_unsafe(key);
        &self.impl_.data[bucket]
    }

    pub fn find(&self, key: &K) -> Option<&K> {
        self.impl_.find(key).map(|bucket| &self.impl_.data[bucket])
    }

    pub fn contains(&self, key: &K) -> bool {
        self.impl_.find(key).is_some()
    }

    pub fn erase(&mut self, key: &K) {
        self.impl_.erase(key);
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

    pub fn iter(&self) -> DenseHashTable2Iter<'_, K> {
        self.impl_.iter()
    }

    /// Mutating a key can invalidate its probe position, matching the C++ API.
    pub fn iter_mut(&mut self) -> DenseHashTable2IterMut<'_, K> {
        self.impl_.iter_mut()
    }
}

impl<K, H, E> PartialEq for DenseHashSet2<K, H, E>
where
    K: Clone + DenseDefault,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default,
{
    fn eq(&self, other: &Self) -> bool {
        self.size() == other.size() && self.iter().all(|key| other.contains(key))
    }
}

impl<K, H, E> Eq for DenseHashSet2<K, H, E>
where
    K: Clone + DenseDefault,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default,
{
}
