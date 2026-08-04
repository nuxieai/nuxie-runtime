use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::sccp::Sccp;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn erase_dead_producer(&mut self, op: BcOp) {
        if op.kind != BcOpKind::Inst {
            return;
        }
        let inst = &self.func().instructions[op.index as usize];
        if self.is_pure_producer(inst.op) && inst.uses.is_empty() {
            self.erase_op(op);
        }
    }
}
