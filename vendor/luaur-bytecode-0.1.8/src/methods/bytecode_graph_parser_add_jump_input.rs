use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::functions::is_fast_call::is_fast_call;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> BytecodeGraphParser<'a> {
    pub fn add_jump_input(&mut self, inst: BcOp, target: i32) {
        let opcode = self.func.instructions[inst.index as usize].op;
        LUAU_ASSERT!(!is_fast_call(opcode));
        if target < 0 {
            LUAU_ASSERT!(opcode == LuauOpcode::LOP_LOADB);
            return;
        }
        let target = target as u32;
        let it = self.block_by_pc.find(&target);
        LUAU_ASSERT!(it.is_some());
        let bc_op = *it.unwrap();
        self.func.add_use_inst(inst, bc_op);
    }
}
