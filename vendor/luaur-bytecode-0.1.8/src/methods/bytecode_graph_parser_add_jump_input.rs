use crate::records::bc_inst::BcInst;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::functions::is_fast_call::is_fast_call;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> BytecodeGraphParser<'a> {
    pub fn add_jump_input(&mut self, inst: *mut BcInst, target: i32) {
        let inst = unsafe { &mut *inst };
        LUAU_ASSERT!(!is_fast_call(inst.op));
        if target < 0 {
            LUAU_ASSERT!(inst.op == LuauOpcode::LOP_LOADB);
            return;
        }
        let target = target as u32;
        let it = self.block_by_pc.find(&target);
        LUAU_ASSERT!(it.is_some());
        let bc_op = *it.unwrap();
        inst.ops.push(bc_op);
    }
}
