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
            {
                let mut phi = self.caller.phi(phi_op);
                phi.operator_deref_mut()
                    .ops
                    .push_back(self.return_ops[idx as usize]);
            }
            self.return_ops[idx as usize] = phi_op;
        } else {
            let mut phi = self.caller.phi(self.return_ops[idx as usize]);
            phi.operator_deref_mut().ops.push_back(op);
        }
    }
}
