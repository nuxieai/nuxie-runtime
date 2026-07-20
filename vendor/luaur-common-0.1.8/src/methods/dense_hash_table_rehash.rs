use crate::macros::luau_assert::LUAU_ASSERT;
use crate::records::dense_hash_table::{DenseEq, DenseHashTable, DenseHasher, ItemInterface};

impl<K, I, Iface, H, E> DenseHashTable<K, I, Iface, H, E>
where
    K: Clone,
    Iface: ItemInterface<K, I>,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default,
{
    pub(crate) fn rehash_stub(&mut self) {
        let new_size = if self.capacity == 0 {
            16
        } else {
            self.capacity * 2
        };

        let mut new_table: DenseHashTable<K, I, Iface, H, E> =
            DenseHashTable::new(self.empty_key.clone(), new_size);

        for i in 0..self.capacity {
            let key = Iface::get_key(&self.data[i]);

            if !self.eq.eq(key, &self.empty_key) {
                let index = new_table.insert_unsafe(key.clone());
                self.data[i] = core::mem::replace(
                    &mut new_table.data[index],
                    core::mem::replace(&mut self.data[i], Iface::make_empty(&self.empty_key)),
                );
            }
        }

        LUAU_ASSERT!(self.count == new_table.count);

        core::mem::swap(&mut self.data, &mut new_table.data);
        core::mem::swap(&mut self.capacity, &mut new_table.capacity);
    }
}
