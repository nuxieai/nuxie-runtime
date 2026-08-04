use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_imm::BcImm;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcFunction {
    pub fn imm<'a>(&'a self, op: BcOp) -> BcRef<'a, BcImm> {
        LUAU_ASSERT!(op.kind == BcOpKind::Imm);
        BcRef {
            vec: &self.immediates,
            op,
        }
    }
}
