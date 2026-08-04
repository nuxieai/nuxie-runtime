use crate::enums::bc_vm_const_kind::BcVmConstKind;
use crate::records::bc_op::BcOp;
use crate::records::bc_vm_const_impl::BcVmConstImpl;

impl BcVmConstImpl {
    pub fn is_arithmetic_constant(&self, vm_const_op: &BcOp) -> bool {
        unsafe { &mut *self.func }.const_op(*vm_const_op).kind == BcVmConstKind::Number
    }
}
