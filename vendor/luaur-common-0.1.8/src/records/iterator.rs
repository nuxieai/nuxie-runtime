//! `DenseHashTable::iterator` — the mutable counterpart of `const_iterator`,
//! yielding `&mut Item` over occupied slots. Reference:
//! `luau/Common/include/Luau/DenseHash.h` (the `iterator` nested type).
//!
//! Unlike the const iterator it walks a `slice::IterMut` directly (a shared
//! borrow of the whole table would conflict with the `&mut Item` it yields), so
//! it carries its own copy of the `empty_key` sentinel and `eq` functor to skip
//! free slots.

use core::marker::PhantomData;

use crate::records::dense_hash_table::{DenseEq, ItemInterface};

pub struct MutIterator<'a, K, I, Iface, E> {
    pub(crate) inner: core::slice::IterMut<'a, I>,
    pub(crate) empty_key: K,
    pub(crate) eq: E,
    pub(crate) _iface: PhantomData<Iface>,
}

impl<'a, K, I, Iface, E> Iterator for MutIterator<'a, K, I, Iface, E>
where
    Iface: ItemInterface<K, I>,
    E: DenseEq<K>,
{
    type Item = &'a mut I;

    fn next(&mut self) -> Option<&'a mut I> {
        for item in self.inner.by_ref() {
            if !self.eq.eq(Iface::get_key(item), &self.empty_key) {
                return Some(item);
            }
        }
        None
    }
}
