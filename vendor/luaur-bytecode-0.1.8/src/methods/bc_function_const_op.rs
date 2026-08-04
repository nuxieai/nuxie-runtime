use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use crate::records::bc_vm_const::BcVmConst;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcFunction {
    pub fn const_op(&mut self, op: BcOp) -> &mut BcVmConst {
        LUAU_ASSERT!(op.kind == BcOpKind::VmConst);
        &mut self.constants[op.index as usize]
    }
}
