use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_block::BcBlock;
use crate::records::bc_function::BcFunction;
use crate::records::bc_inst_helper::BcInstHelper;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcInstHelper<'_> {
    pub fn get_block(&mut self, input_idx: u32) -> BcRef<'_, BcBlock> {
        let block_op = self.get_bc_op(input_idx);
        LUAU_ASSERT!(block_op.kind == BcOpKind::Block);
        self.graph.block(block_op)
    }
}
