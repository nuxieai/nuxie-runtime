use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use crate::records::bc_proj::BcProj;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcFunction {
    pub fn proj_op(&mut self, op: BcOp) -> &mut BcProj {
        LUAU_ASSERT!(op.kind == BcOpKind::Proj);
        &mut self.projections[op.index as usize]
    }
}
