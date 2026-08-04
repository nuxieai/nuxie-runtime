use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;

impl BcFunction {
    pub fn erase_op(&mut self, op: BcOp) {
        let block = self.instructions[op.index as usize].block;
        self.blocks[block.index as usize]
            .ops
            .retain(|candidate| *candidate != op);
    }
}
