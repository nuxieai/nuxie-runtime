use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use crate::type_aliases::reg::Reg;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> BytecodeGraphParser<'a> {
    pub fn add_vm_reg_input(&mut self, inst: *mut BcInst, reg: Reg) {
        let inst = unsafe { &mut *inst };
        let source = self.find_producer_bc_op_reg(self.current_block, reg);
        if source.is_none() && crate::methods::bytecode_graph_parser_is_unreachable::bytecode_graph_parser_is_unreachable(self, self.current_block) {
            inst.ops.push_back(BcOp::bc_op_bc_op_kind_u32(BcOpKind::VmReg, reg as u32));
            return;
        }
        LUAU_ASSERT!(source.is_some());
        inst.ops.push_back(source.unwrap());
    }
}
