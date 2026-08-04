use crate::records::string_ref::StringRef;
use crate::records::string_ref_hash::StringRefHash;
use luaur_common::functions::hash_range::hashRange;

impl StringRefHash {
    #[allow(non_snake_case)]
    pub fn operator_call(&self, v: &StringRef) -> usize {
        unsafe { hashRange(v.data, v.length) }
    }
}
