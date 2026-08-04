use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_function::VmConst;
use crate::records::bc_inst_helper::BcInstHelper;
use crate::records::bc_ref::BcRef;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BcInstHelper<'_> {
    pub fn get_vm_const(&mut self, input_idx: u32) -> BcRef<'_, VmConst> {
        let const_op = self.inst.operator_deref().ops[input_idx as usize];
        LUAU_ASSERT!(const_op.kind == BcOpKind::VmConst);
        self.graph.vm_const(const_op)
    }
}
