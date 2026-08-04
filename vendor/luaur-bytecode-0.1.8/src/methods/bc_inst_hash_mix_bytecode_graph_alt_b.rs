use crate::records::bc_inst_hash::BcInstHash;
use crate::records::bc_op::BcOp;

impl BcInstHash {
    pub fn mix_u32_bc_op(h: u32, op: BcOp) -> u32 {
        let mut k: u32 = 0;
        unsafe {
            core::ptr::copy_nonoverlapping(
                &op as *const BcOp as *const u8,
                &mut k as *mut u32 as *mut u8,
                core::mem::size_of::<BcOp>(),
            );
        }
        Self::mix_u32_u32(h, k)
    }
}
