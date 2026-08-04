use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use crate::records::bc_phi::BcPhi;
use crate::records::call_inliner::CallInliner;

impl<'a> CallInliner<'a> {
    pub fn set_return_op(&mut self, idx: u32, op: BcOp) {
        if (idx as usize) >= self.return_ops.len() {
            self.return_ops.resize(idx as usize + 1, BcOp::new());
        }

        if self.return_ops[idx as usize].kind == BcOpKind::None {
            self.return_ops[idx as usize] = op;
            return;
        }

        if self.return_ops[idx as usize].kind != BcOpKind::Phi {
            let phi_op = self.caller.add_phi();
            self.caller
                .add_use_phi(phi_op, self.return_ops[idx as usize]);
            self.return_ops[idx as usize] = phi_op;
        } else {
            let mut phi = self.caller.phi(self.return_ops[idx as usize]);
            let exists = phi.operator_deref().ops.iter().any(|phi_op| *phi_op == op);
            if !exists {
                self.caller
                    .add_use_phi(self.return_ops[idx as usize], op);
            }
        }
    }
}
