//! Presence-bit dense hash table record from `Luau/DenseHash2.h`.

use alloc::vec::Vec;
use core::marker::PhantomData;

use crate::records::bit_set::BitSet;

pub struct DenseHashTable2<K, I, Iface, H, E> {
    pub(crate) data: Vec<Option<I>>,
    pub(crate) used_table: BitSet,
    pub(crate) capacity: usize,
    pub(crate) count: usize,
    pub(crate) hash_shift: u8,
    pub(crate) hasher: H,
    pub(crate) eq: E,
    pub(crate) _key: PhantomData<K>,
    pub(crate) _iface: PhantomData<Iface>,
}
