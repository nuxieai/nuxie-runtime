use crate::macros::luau_assert::LUAU_ASSERT;
use crate::records::dense_hash_table::{
    DenseDefault, DenseEq, DenseEqDefault, DenseHashTable, DenseHasher, ItemInterface,
};

pub fn dense_hash_table_insert_unsafe<K, I, Iface, H, E>(
    table: &mut DenseHashTable<K, I, Iface, H, E>,
    key: K,
) -> *mut I
where
    K: Clone,
    Iface: ItemInterface<K, I>,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default,
{
    // It is invalid to insert empty_key into the table since it acts as a "entry does not exist" marker
    LUAU_ASSERT!(!table.eq.eq(&key, &table.empty_key));

    let hash_mod = table.capacity - 1;
    let mut bucket = table.hasher.hash(&key) & hash_mod;

    for probe in 0..=hash_mod {
        let probe_item = &mut table.data[bucket];

        // Element does not exist, insert here
        if table.eq.eq(Iface::get_key(probe_item), &table.empty_key) {
            Iface::set_key(probe_item, key);
            table.count += 1;
            return probe_item as *mut I;
        }

        // Element already exists
        if table.eq.eq(Iface::get_key(probe_item), &key) {
            return probe_item as *mut I;
        }

        // Hash collision, quadratic probing
        bucket = (bucket + probe + 1) & hash_mod;
    }

    // Hash table is full - this should not happen
    LUAU_ASSERT!(false);
    core::ptr::null_mut()
}
