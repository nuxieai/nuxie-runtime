//! `DenseHashTable::const_iterator` — a forward iterator over occupied slots,
//! yielding `&Item`. Reference: `luau/Common/include/Luau/DenseHash.h` (the
//! `const_iterator` nested type). The map/set wrappers seed it with
//! `first_occupied()` and adapt the yielded `&Item` into `&Key`/`(&Key, &Value)`.

use crate::records::dense_hash_table::{DenseEq, DenseHashTable, DenseHasher, ItemInterface};

pub struct ConstIterator<'a, K, I, Iface, H, E> {
    pub(crate) table: &'a DenseHashTable<K, I, Iface, H, E>,
    pub(crate) index: usize,
}

impl<'a, K, I, Iface, H, E> Iterator for ConstIterator<'a, K, I, Iface, H, E>
where
    K: Clone,
    Iface: ItemInterface<K, I>,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default,
{
    type Item = &'a I;

    fn next(&mut self) -> Option<&'a I> {
        if self.index >= self.table.capacity {
            return None;
        }
        let item = &self.table.data[self.index];
        self.index = self.table.next_occupied(self.index);
        Some(item)
    }
}
