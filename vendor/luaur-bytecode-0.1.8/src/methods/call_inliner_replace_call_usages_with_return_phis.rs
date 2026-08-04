use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::call_inliner::CallInliner;

impl<'a> CallInliner<'a> {
    pub fn replace_call_usages_with_return_phis(&mut self) {
        for i in 0..self.caller_inst_size_before_inline {
            // To avoid simultaneous mutable borrow of `self` and `self.caller`,
            // we must swap the ops out or access them in a way that doesn't hold a borrow
            // across the method call. Since BcOps is a SmallVector (which is Clone),
            // we clone the ops, modify them, and put them back.
            let mut ops = self.caller.instructions[i as usize].ops.clone();
            self.replace_call_usages_in_ops(&mut ops);
            self.caller.instructions[i as usize].ops = ops;
        }

        for i in 0..self.caller.phis.len() {
            let candidate = BcOp::bc_op_bc_op_kind_u32(BcOpKind::Phi, i as u32);

            let mut found = false;
            for op in &self.return_ops {
                if op.operator_eq(&candidate) {
                    found = true;
                    break;
                }
            }

            if !found {
                let mut ops = self.caller.phis[i].ops.clone();
                self.replace_call_usages_in_ops(&mut ops);
                self.caller.phis[i].ops = ops;
            }
        }
    }
}
