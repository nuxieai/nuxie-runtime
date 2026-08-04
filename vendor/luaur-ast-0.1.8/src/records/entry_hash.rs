use crate::records::entry::Entry;
use luaur_common::records::dense_hash_table::DenseHasher;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct EntryHash;

// The `DenseHashSet<Entry, EntryHash>` hash functor. `EntryHash::operator_call`
// (the FNV-1a over the name bytes) lives in its own method item; this bridges it
// to the `DenseHasher` trait the table is generic over.
impl DenseHasher<Entry> for EntryHash {
    fn hash(&self, key: &Entry) -> usize {
        self.operator_call(key)
    }
}
