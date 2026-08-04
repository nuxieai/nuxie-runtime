use crate::records::bc_op::BcOp;
use crate::records::bc_op_hash::BcOpHash;
use core::hash::{Hash, Hasher};
use luaur_common::records::dense_hash_table::DenseHasher;

impl BcOpHash {
    pub fn operator_call(&self, p: &BcOp) -> usize {
        let mut res: usize = 0;
        let size = core::mem::size_of::<BcOp>().min(core::mem::size_of::<usize>());
        unsafe {
            core::ptr::copy_nonoverlapping(
                p as *const BcOp as *const u8,
                &mut res as *mut usize as *mut u8,
                size,
            );
        }
        res
    }
}

impl std::hash::BuildHasher for BcOpHash {
    type Hasher = std::collections::hash_map::DefaultHasher;
    fn build_hasher(&self) -> Self::Hasher {
        std::collections::hash_map::DefaultHasher::new()
    }
}

impl DenseHasher<BcOp> for BcOpHash {
    fn hash(&self, key: &BcOp) -> usize {
        self.operator_call(key)
    }
}
