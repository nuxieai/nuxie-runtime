use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> BytecodeGraphParser<'a> {
    pub fn add_vm_const_input(&mut self, inst: BcOp, idx: u32) {
        LUAU_ASSERT!(idx < self.func.constants.len() as u32);
        self.func
            .add_use_inst(inst, BcOp::bc_op_bc_op_kind_u32(BcOpKind::VmConst, idx));
    }
}
