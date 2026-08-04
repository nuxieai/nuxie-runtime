use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use crate::records::bc_phi::BcPhi;
use crate::records::bc_ref::BcRef;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcFunction {
    pub fn phi<'a>(&'a self, op: BcOp) -> BcRef<'a, BcPhi> {
        LUAU_ASSERT!(op.kind == BcOpKind::Phi);
        BcRef {
            vec: &self.phis,
            op,
        }
    }
}
