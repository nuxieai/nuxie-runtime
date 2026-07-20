use crate::macros::luau_assert::LUAU_ASSERT;
use crate::records::dense_hash_table::{DenseEq, DenseHashTable, DenseHasher, ItemInterface};
use alloc::vec::Vec;
use core::marker::PhantomData;

impl<K, I, Iface, H, E> DenseHashTable<K, I, Iface, H, E>
where
    K: Clone,
    Iface: ItemInterface<K, I>,
    H: DenseHasher<K> + Default,
    E: DenseEq<K> + Default,
{
    #[allow(non_snake_case)]
    pub fn dense_hash_table_key_usize(empty_key: K, buckets: usize) -> Self {
        let hasher = H::default();
        let eq = E::default();

        // validate that equality operator is at least somewhat functional
        LUAU_ASSERT!(eq.eq(&empty_key, &empty_key));
        // buckets has to be power-of-two or zero
        LUAU_ASSERT!((buckets & (buckets.wrapping_sub(1))) == 0);

        let mut data = Vec::new();
        let mut capacity = 0;

        if buckets > 0 {
            data.reserve_exact(buckets);
            for _ in 0..buckets {
                data.push(Iface::make_empty(&empty_key));
            }
            capacity = buckets;
        }

        Self {
            data,
            capacity,
            count: 0,
            empty_key,
            hasher,
            eq,
            _iface: PhantomData,
        }
    }
}
