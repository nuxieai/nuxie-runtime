use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcFunction {
    pub fn inst<'a>(&'a self, op: BcOp) -> BcRef<'a, BcInst> {
        LUAU_ASSERT!(op.kind == BcOpKind::Inst);
        BcRef {
            vec: &self.instructions,
            op,
        }
    }
}
