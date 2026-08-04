use crate::enums::bc_vm_const_kind::BcVmConstKind;
use crate::records::bc_op::BcOp;
use crate::records::bc_vm_const_impl::BcVmConstImpl;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcVmConstImpl {
    pub fn as_number(&self, vm_const_op: &BcOp) -> f64 {
        let vm_const = *unsafe { &mut *self.func }.const_op(*vm_const_op);
        LUAU_ASSERT!(vm_const.kind == BcVmConstKind::Number);
        unsafe { vm_const.value.valueNumber }
    }
}
