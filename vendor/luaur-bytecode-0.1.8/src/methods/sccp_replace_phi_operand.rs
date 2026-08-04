use crate::records::bc_op::BcOp;
use crate::records::sccp::Sccp;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn replace_phi_operand(&mut self, phi_op: BcOp, old_op: BcOp, new_op: BcOp) {
        for operand in self.func_mut().phis[phi_op.index as usize].ops.iter_mut() {
            if *operand == old_op {
                *operand = new_op;
            }
        }
    }
}
