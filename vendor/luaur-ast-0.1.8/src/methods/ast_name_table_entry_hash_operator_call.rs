use crate::records::entry::Entry;
use crate::records::entry_hash::EntryHash;

impl EntryHash {
    #[allow(non_snake_case)]
    pub fn operator_call(&self, e: &Entry) -> usize {
        // FNV1a
        let mut hash: u32 = 2166136261;

        for i in 0..e.length as usize {
            unsafe {
                let byte = *e.value.value.add(i) as u8;
                hash ^= byte as u32;
                hash = hash.wrapping_mul(16777619);
            }
        }

        hash as usize
    }
}

#[allow(non_snake_case)]
pub fn ast_name_table_entry_hash_operator_call(this: &EntryHash, e: &Entry) -> usize {
    this.operator_call(e)
}
