use crate::records::dense_hash_map::DenseHashMap;
use crate::records::dense_hash_table::{DenseDefault, DenseEq, DenseHasher};

impl<K, V, H, E> DenseHashMap<K, V, H, E>
where
    K: Clone,
    V: DenseDefault,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default + Clone,
{
    // The C++ operator[] logic is already implemented as get_or_insert in the record file.
    // This file provides the operator-style free function alias for compatibility.
}

#[allow(non_snake_case)]
pub fn dense_hash_map_operator_index<K, V, H, E>(
    map: &mut DenseHashMap<K, V, H, E>,
    key: K,
) -> &mut V
where
    K: Clone,
    V: DenseDefault,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default + Clone,
{
    map.get_or_insert(key)
}
