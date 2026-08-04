use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;

impl BcFunction {
    pub fn record_use(&mut self, used_op: BcOp, user: BcOp) {
        if used_op.kind == BcOpKind::Inst {
            self.inst_op(used_op).uses.push(user);
        } else if used_op.kind == BcOpKind::Phi {
            self.phi_op(used_op).uses.push(user);
        }
    }
}
