use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcFunction {
    pub fn inst_op(&mut self, op: BcOp) -> &mut BcInst {
        LUAU_ASSERT!(op.kind == BcOpKind::Inst);
        &mut self.instructions[op.index as usize]
    }
}
