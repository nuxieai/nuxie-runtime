use crate::records::bytecode_builder::BytecodeBuilder;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub fn undo_emit(&mut self, op: LuauOpcode) {
        LUAU_ASSERT!(!self.insns.is_empty());
        LUAU_ASSERT!((self.insns[self.insns.len() - 1] & 0xff) == op as u32);

        self.insns.pop();
        self.lines.pop();
    }
}
