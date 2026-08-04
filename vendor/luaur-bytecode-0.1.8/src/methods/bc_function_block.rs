use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_block::BcBlock;
use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcFunction {
    pub fn block<'a>(&'a self, op: BcOp) -> BcRef<'a, BcBlock> {
        LUAU_ASSERT!(op.kind == BcOpKind::Block);
        BcRef {
            vec: &self.blocks,
            op,
        }
    }
}
