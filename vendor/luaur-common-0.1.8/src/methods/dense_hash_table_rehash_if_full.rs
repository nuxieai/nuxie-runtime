use crate::records::dense_hash_table::DenseHashTable;

impl<K, I, Iface, H, E> DenseHashTable<K, I, Iface, H, E>
where
    K: Clone,
    Iface: crate::records::dense_hash_table::ItemInterface<K, I>,
    H: crate::records::dense_hash_table::DenseHasher<K> + Default,
    E: crate::records::dense_hash_table::DenseEq<K> + Default,
{
    pub(crate) fn dense_hash_table_rehash_if_full(&mut self, key: &K) {
        if self.count >= self.capacity * 3 / 4 && self.find(key).is_none() {
            DenseHashTable::<K, I, Iface, H, E>::rehash(self);
        }
    }
}
