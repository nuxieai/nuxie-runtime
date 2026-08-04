use crate::records::bc_inst_helper::BcInstHelper;
use crate::records::bc_op::BcOp;

impl BcInstHelper<'_> {
    pub fn slice_inputs(&self, start_from: u32) -> Vec<BcOp> {
        let ops = &self.inst.operator_deref().ops;
        let start = start_from as usize;
        let mut result = Vec::new();
        result.reserve(ops.len() - start);
        for i in start..ops.len() {
            result.push(ops[i]);
        }
        result
    }
}
