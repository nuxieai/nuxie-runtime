use crate::macros::luau_assert::LUAU_ASSERT;
use crate::records::dense_hash_table::DenseHashTable;
use crate::records::dense_hash_table::ItemInterface;

/// Finds an item by key in the hash table using quadratic probing.
/// Returns a raw pointer to the item if found, or `null` if not found or if the key equals the empty key.
pub fn dense_hash_table_find<K, I, Iface, H, E>(
    table: &DenseHashTable<K, I, Iface, H, E>,
    key: &K,
) -> *const I
where
    K: Clone,
    Iface: ItemInterface<K, I>,
    H: crate::records::dense_hash_table::DenseHasher<K> + Default,
    E: crate::records::dense_hash_table::DenseEq<K> + Default,
{
    if table.count == 0 {
        return core::ptr::null();
    }
    if table.eq.eq(key, &table.empty_key) {
        return core::ptr::null();
    }

    let hashmod = table.capacity.wrapping_sub(1);
    let mut bucket = (table.hasher.hash(key) & hashmod) as usize;

    for probe in 0..=hashmod as usize {
        let probe_item = unsafe { table.data.get_unchecked(bucket) };
        let probe_key = Iface::get_key(probe_item);

        if table.eq.eq(probe_key, key) {
            return probe_item as *const I;
        }

        if table.eq.eq(probe_key, &table.empty_key) {
            return core::ptr::null();
        }

        bucket = (bucket.wrapping_add(probe).wrapping_add(1)) & hashmod as usize;
    }

    LUAU_ASSERT!(false);
    core::ptr::null()
}
