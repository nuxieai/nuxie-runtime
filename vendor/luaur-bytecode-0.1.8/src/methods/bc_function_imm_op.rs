use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_imm::BcImm;
use crate::records::bc_op::BcOp;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcFunction {
    pub fn imm_op(&mut self, op: BcOp) -> &mut BcImm {
        LUAU_ASSERT!(op.kind == BcOpKind::Imm);
        &mut self.immediates[op.index as usize]
    }
}
