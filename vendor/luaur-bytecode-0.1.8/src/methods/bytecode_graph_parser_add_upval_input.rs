use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> BytecodeGraphParser<'a> {
    pub fn add_upval_input(&mut self, inst: *mut BcInst, idx: u32) {
        let inst = unsafe { &mut *inst };
        LUAU_ASSERT!(idx < self.func.nups.into());
        inst.ops.push_back(BcOp::bc_op_bc_op_kind_u32(
            crate::enums::bc_op_kind::BcOpKind::VmUpvalue,
            idx,
        ));
    }
}
