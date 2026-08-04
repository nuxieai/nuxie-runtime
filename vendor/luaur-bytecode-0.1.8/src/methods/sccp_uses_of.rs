use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::sccp::Sccp;
use alloc::vec::Vec;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub(crate) fn uses_of(&self, op: BcOp) -> Vec<BcOp> {
        if op.kind == BcOpKind::Inst {
            self.func().instructions[op.index as usize].uses.clone()
        } else {
            LUAU_ASSERT!(op.kind == BcOpKind::Phi);
            self.func().phis[op.index as usize].uses.clone()
        }
    }
}
