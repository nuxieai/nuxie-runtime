use crate::records::bc_op::BcOp;
use crate::records::sccp::Sccp;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn erase_op(&mut self, op: BcOp) {
        let block_op = self.func().instructions[op.index as usize].block;
        self.func_mut().blocks[block_op.index as usize]
            .ops
            .retain(|candidate| *candidate != op);
    }
}
