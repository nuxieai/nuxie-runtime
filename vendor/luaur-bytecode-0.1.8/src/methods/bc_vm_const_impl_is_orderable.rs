use crate::enums::bc_vm_const_kind::BcVmConstKind;
use crate::records::bc_op::BcOp;
use crate::records::bc_vm_const_impl::BcVmConstImpl;

impl BcVmConstImpl {
    pub fn is_orderable(&self, vm_const_op: &BcOp) -> bool {
        let vm_const = *unsafe { &mut *self.func }.const_op(*vm_const_op);
        matches!(
            vm_const.kind,
            BcVmConstKind::Number | BcVmConstKind::Integer | BcVmConstKind::String
        )
    }
}
