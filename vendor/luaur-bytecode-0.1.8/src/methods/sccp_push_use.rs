use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::sccp::Sccp;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub(crate) fn push_use(&mut self, op: BcOp, user: BcOp) {
        if op.kind == BcOpKind::Inst {
            self.func_mut().instructions[op.index as usize]
                .uses
                .push(user);
        } else {
            LUAU_ASSERT!(op.kind == BcOpKind::Phi);
            self.func_mut().phis[op.index as usize].uses.push(user);
        }
    }
}
