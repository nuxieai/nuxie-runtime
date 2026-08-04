use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::call_inliner::CallInliner;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> CallInliner<'a> {
    pub fn map_block_op(&mut self, target_block: BcOp) -> BcOp {
        LUAU_ASSERT!(target_block.kind == BcOpKind::Block);
        BcOp::bc_op_bc_op_kind_u32(
            BcOpKind::Block,
            self.caller_blocks_size_before_inline + target_block.index,
        )
    }
}
