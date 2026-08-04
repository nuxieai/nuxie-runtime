use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use crate::records::bc_phi::BcPhi;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcFunction {
    pub fn phi_op(&mut self, op: BcOp) -> &mut BcPhi {
        LUAU_ASSERT!(op.kind == BcOpKind::Phi);
        &mut self.phis[op.index as usize]
    }
}
