use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub fn call_inliner_map_vm_const_op(
    caller_vm_const_size_before_inline: u32,
    target_vm_const: BcOp,
) -> BcOp {
    LUAU_ASSERT!(target_vm_const.kind == BcOpKind::VmConst);
    BcOp {
        kind: BcOpKind::VmConst,
        index: caller_vm_const_size_before_inline + target_vm_const.index,
    }
}
