use crate::records::bytecode_builder::BytecodeBuilder;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl BytecodeBuilder {
    pub fn emit_ad(&mut self, op: LuauOpcode, a: u8, d: i16) {
        let insn = op as u32 | ((a as u32) << 8) | ((d as u16 as u32) << 16);

        self.insns.push(insn);
        self.lines.push(self.debug_line);
    }
}
