use crate::records::dense_hash_map::DenseHashMap;
use crate::records::dense_hash_table::{DenseDefault, DenseEq, DenseHasher};

impl<K, V, H, E> DenseHashMap<K, V, H, E>
where
    K: Clone,
    V: DenseDefault,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default + Clone,
{
    #[allow(non_snake_case)]
    pub fn try_insert_mut(&mut self, key: K, value: V) -> (&mut V, bool) {
        self.impl_.dense_hash_table_rehash_if_full(&key);

        let before = self.impl_.size();
        // In DenseHashMap, the internal table stores std::pair<Key, Value>, which is (K, V) in Rust.
        let slot = crate::methods::dense_hash_table_insert_unsafe::dense_hash_table_insert_unsafe(
            &mut self.impl_,
            key.clone(),
        );

        // Value is fresh if container count has increased
        let fresh = self.impl_.size() > before;

        if fresh {
            unsafe {
                // The slot points to the pair (K, V). We update the value part (second).
                (*slot).1 = value;
            }
        }

        // SAFETY: slot points to a valid entry in the table.
        // We return a reference to the value (the second element of the pair).
        let ref_val = unsafe { &mut (*slot).1 };
        (ref_val, fresh)
    }
}
