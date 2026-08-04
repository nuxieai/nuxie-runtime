use crate::records::bc_op::BcOp;
use crate::records::bc_vm_const_impl::BcVmConstImpl;

impl BcVmConstImpl {
    pub fn kind_equals(&self, lhs_op: &BcOp, rhs_op: &BcOp) -> bool {
        let func = unsafe { &mut *self.func };
        let lhs = *func.const_op(*lhs_op);
        let rhs = *func.const_op(*rhs_op);
        lhs.kind == rhs.kind
    }
}
