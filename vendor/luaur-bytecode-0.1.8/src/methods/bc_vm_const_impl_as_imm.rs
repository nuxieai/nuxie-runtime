use crate::records::bc_imm::BcImm;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;
use crate::records::bc_vm_const_impl::BcVmConstImpl;

impl BcVmConstImpl {
    pub fn as_imm(&self, op: BcOp) -> BcRef<'_, BcImm> {
        unsafe { (&*self.func).imm(op) }
    }
}
