use crate::records::compiler::Compiler;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn emit_load_k(&mut self, target: u8, cid: i32) {
        LUAU_ASSERT!(cid >= 0);

        let bytecode = unsafe { &mut *self.bytecode };

        if cid < 32768 {
            bytecode.emit_ad(LuauOpcode::LOP_LOADK, target, cid as i16);
        } else {
            bytecode.emit_ad(LuauOpcode::LOP_LOADKX, target, 0);
            bytecode.emit_aux(cid as u32);
        }
    }
}
