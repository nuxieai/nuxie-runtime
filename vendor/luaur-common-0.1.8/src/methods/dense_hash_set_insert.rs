use crate::records::dense_hash_set::DenseHashSet;
use crate::records::dense_hash_table::{DenseEq, DenseHasher};

#[allow(non_snake_case)]
pub fn dense_hash_set_insert<K, H, E>(set: &mut DenseHashSet<K, H, E>, key: K) -> &K
where
    K: Clone,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default,
{
    set.insert(key)
}
