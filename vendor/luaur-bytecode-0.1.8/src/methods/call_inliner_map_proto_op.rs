use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::call_inliner::CallInliner;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> CallInliner<'a> {
    pub fn map_proto_op(&self, target_proto_op: BcOp) -> BcOp {
        LUAU_ASSERT!(target_proto_op.kind == BcOpKind::VmProto);
        BcOp::bc_op_bc_op_kind_u32(
            BcOpKind::VmProto,
            self.caller_proto_size_before_inline + target_proto_op.index,
        )
    }
}
