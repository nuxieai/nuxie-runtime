use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_block::BcBlock;
use crate::records::bc_op::BcOp;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcBlock {
    pub fn append_instruction(&mut self, inst: BcOp) {
        LUAU_ASSERT!(inst.kind == BcOpKind::Inst);
        self.ops.push_back(inst);
    }
}
