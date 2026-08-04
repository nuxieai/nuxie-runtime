use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::call_inliner::CallInliner;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> CallInliner<'a> {
    pub(crate) fn map_inst_op(&mut self, target_inst: BcOp) -> BcOp {
        LUAU_ASSERT!(target_inst.kind == BcOpKind::Inst);
        BcOp::bc_op_bc_op_kind_u32(
            BcOpKind::Inst,
            self.caller_inst_size_before_inline + target_inst.index,
        )
    }
}
