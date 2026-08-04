use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_block::BcBlock;
use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use luaur_common::LUAU_ASSERT;

impl BcFunction {
    pub fn block_op(&mut self, op: BcOp) -> &mut BcBlock {
        LUAU_ASSERT!(op.kind == BcOpKind::Block);
        &mut self.blocks[op.index as usize]
    }
}
