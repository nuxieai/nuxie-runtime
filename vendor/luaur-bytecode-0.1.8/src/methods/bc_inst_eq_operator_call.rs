use crate::records::bc_inst::BcInst;
use crate::records::bc_inst_eq::BcInstEq;

impl BcInstEq {
    #[allow(non_snake_case)]
    pub fn call(&self, a: &BcInst, b: &BcInst) -> bool {
        if a.op != b.op || a.ops.len() != b.ops.len() {
            return false;
        }
        for i in 0..a.ops.len() {
            if a.ops[i] != b.ops[i] {
                return false;
            }
        }
        true
    }
}
