use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use crate::type_aliases::reg::Reg;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> BytecodeGraphParser<'a> {
    pub fn add_vm_reg_input(&mut self, inst: BcOp, reg: Reg) {
        let source = self.read_variable(self.current_block, reg);
        if source.is_none() && crate::methods::bytecode_graph_parser_is_unreachable::bytecode_graph_parser_is_unreachable(self, self.current_block) {
            self.func.add_use_inst(inst, BcOp::bc_op_bc_op_kind_u32(BcOpKind::VmReg, reg as u32));
            return;
        }
        LUAU_ASSERT!(source.is_some());
        self.func.add_use_inst(inst, source.unwrap());
    }
}
