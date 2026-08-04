use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use crate::records::bc_proj::BcProj;
use crate::records::bc_ref::BcRef;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcFunction {
    pub fn proj<'a>(&'a self, op: BcOp) -> BcRef<'a, BcProj> {
        LUAU_ASSERT!(op.kind == BcOpKind::Proj);
        BcRef {
            vec: &self.projections,
            op,
        }
    }
}
