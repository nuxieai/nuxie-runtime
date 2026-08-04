use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use crate::type_aliases::instruction::Instruction;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::functions::get_jump_target::getJumpTarget;
use luaur_common::macros::luau_insn_op::LUAU_INSN_OP;

impl<'a> BytecodeGraphParser<'a> {
    pub fn is_jump_trampoline(&self, pc: u32, code: *const Instruction, codesize: u32) -> bool {
        let op0: LuauOpcode =
            unsafe { std::mem::transmute((LUAU_INSN_OP(*code.add(pc as usize)) & 0xff) as u8) };
        if op0 != LuauOpcode::LOP_JUMP {
            return false;
        }

        if pc + 1 >= codesize {
            return false;
        }

        let op1: LuauOpcode = unsafe {
            std::mem::transmute((LUAU_INSN_OP(*code.add((pc + 1) as usize)) & 0xff) as u8)
        };
        if op1 != LuauOpcode::LOP_JUMPX {
            return false;
        }

        let target = unsafe { getJumpTarget(*code.add((pc + 2) as usize), pc + 2) as u32 };
        target == pc + 1
    }
}
