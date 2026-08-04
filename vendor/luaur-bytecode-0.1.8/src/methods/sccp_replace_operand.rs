use crate::records::bc_op::BcOp;
use crate::records::sccp::Sccp;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn replace_operand(&mut self, inst_op: BcOp, old_op: BcOp, new_op: BcOp) {
        for operand in self.func_mut().instructions[inst_op.index as usize]
            .ops
            .iter_mut()
        {
            if *operand == old_op {
                *operand = new_op;
            }
        }
    }
}
