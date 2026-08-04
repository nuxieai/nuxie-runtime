use crate::records::bc_inst::BcInst;
use crate::records::bc_inst_hash::BcInstHash;
use crate::records::bc_op::BcOp;

impl BcInstHash {
    pub fn call(&self, key: &BcInst) -> usize {
        // MurmurHash2 unrolled (faithful to BytecodeGraph.h `BcInstHash::operator()`).
        let mut h: u32 = 25;

        h = Self::mix_u32_u32(h, key.op as u32);
        for i in 0..7 {
            let op_val = if i < key.ops.len() {
                key.ops[i]
            } else {
                BcOp::new()
            };
            h = Self::mix_u32_bc_op(h, op_val);
        }

        // MurmurHash2 tail
        h ^= h >> 13;
        h = h.wrapping_mul(Self::M);
        h ^= h >> 15;

        h as usize
    }
}
