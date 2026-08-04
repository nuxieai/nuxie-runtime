use crate::enums::bc_vm_const_kind::BcVmConstKind;
use crate::functions::find_or_add_const::find_or_add_const;
use crate::records::bc_op::BcOp;
use crate::records::bc_vm_const::BcVmConst;
use crate::records::bc_vm_const_impl::BcVmConstImpl;

impl BcVmConstImpl {
    pub fn make_nil(&self) -> BcOp {
        let mut result = BcVmConst::new();
        result.kind = BcVmConstKind::Nil;
        find_or_add_const(unsafe { &mut *self.func }, &result)
    }
}
