use core::slice;

use crate::records::bit_set::BitSet;

pub struct DenseHashTable2ConstIterator<'a, I> {
    pub(crate) used_table: &'a BitSet,
    pub(crate) inner: core::iter::Enumerate<slice::Iter<'a, Option<I>>>,
}
