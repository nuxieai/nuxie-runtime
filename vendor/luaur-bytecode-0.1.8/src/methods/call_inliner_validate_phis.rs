use crate::enums::bc_op_kind::BcOpKind;
use crate::records::call_inliner::CallInliner;

impl CallInliner<'_> {
    pub fn validate_phis(&self) -> bool {
        for phi in &self.caller.phis {
            for op in &phi.ops {
                luaur_common::LUAU_ASSERT!(
                    op.kind == BcOpKind::Inst
                        || op.kind == BcOpKind::VmReg
                        || op.kind == BcOpKind::Proj
                        || op.kind == BcOpKind::Phi
                );
            }
        }
        true
    }
}
