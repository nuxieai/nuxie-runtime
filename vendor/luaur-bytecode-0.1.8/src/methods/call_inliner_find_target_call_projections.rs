use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use crate::records::bc_phi::BcPhi;
use crate::records::bc_proj::BcProj;
use crate::records::call_inliner::CallInliner;

impl<'a> CallInliner<'a> {
    pub fn find_target_call_projections(&mut self) {
        for i in 0..self.caller.projections.len() {
            let proj = self.caller.projections[i as usize];
            if proj.op == self.call.op() {
                let proj_op =
                    BcOp::bc_op_bc_op_kind_u32(crate::enums::bc_op_kind::BcOpKind::Proj, i as u32);
                if self.call_projections.contains(&proj_op) {
                    continue;
                }
                if (proj.index as usize) >= self.return_ops.len() {
                    self.return_ops.resize(proj.index as usize + 1, BcOp::new());
                }
                let phi_op = self.caller.add_phi();
                let mut phi = self.caller.phi(phi_op);
                phi.operator_deref_mut().ops.push_back(proj_op);
                self.call_projections.insert(proj_op);
                self.return_ops[proj.index as usize] = phi_op;
            }
        }
    }
}
