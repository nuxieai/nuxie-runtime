use crate::records::bytecode_builder::BytecodeBuilder;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl BytecodeBuilder {
    pub fn emit_e(&mut self, op: LuauOpcode, e: i32) {
        let insn = (op as u32) | ((e as u32) << 8);

        self.insns.push(insn);
        self.lines.push(self.debug_line);
    }
}
