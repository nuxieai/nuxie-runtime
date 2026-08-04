use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::sccp::Sccp;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn erase_use(&mut self, user_op: BcOp, used_op: BcOp) {
        let uses = if used_op.kind == BcOpKind::Inst {
            &mut self.func_mut().instructions[used_op.index as usize].uses
        } else if used_op.kind == BcOpKind::Phi {
            &mut self.func_mut().phis[used_op.index as usize].uses
        } else {
            return;
        };
        uses.retain(|op| *op != user_op);
    }
}
