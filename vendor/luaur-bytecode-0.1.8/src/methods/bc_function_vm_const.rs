use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::{BcFunction, VmConst};
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcFunction {
    pub fn vm_const<'a>(&'a self, op: BcOp) -> BcRef<'a, VmConst> {
        LUAU_ASSERT!(op.kind == BcOpKind::VmConst);
        BcRef {
            vec: &self.constants,
            op,
        }
    }
}
