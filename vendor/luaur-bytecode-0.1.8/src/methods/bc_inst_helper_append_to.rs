use crate::records::bc_inst_helper::BcInstHelper;
use crate::records::bc_op::BcOp;

impl BcInstHelper<'_> {
    pub fn append_to(&mut self, block: BcOp) {
        unsafe {
            (*self.inst.operator_arrow()).block = block;
        }
        let block_mut = self.graph.block_op(block);
        block_mut.append_instruction(self.inst.op);
    }
}
