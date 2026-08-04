use crate::records::allocator::Allocator;
use crate::records::entry::Entry;
use crate::records::entry_hash::EntryHash;
use luaur_common::records::dense_hash_set::DenseHashSet;

// `Entry` and `EntryHash` are their own record items (`records::entry`,
// `records::entry_hash`); the table just stores them.
#[repr(C)]
#[derive(Debug)]
pub struct AstNameTable {
    pub(crate) data: DenseHashSet<Entry, EntryHash>,
    pub(crate) allocator: *mut Allocator,
}
