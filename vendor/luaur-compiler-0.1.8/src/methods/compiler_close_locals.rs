use crate::records::compiler::Compiler;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Compiler {
    pub fn close_locals(&mut self, start: usize) {
        LUAU_ASSERT!(start <= self.local_stack.len());
        let mut captured = false;
        let mut capture_reg = 255u8;
        for i in start..self.local_stack.len() {
            let local_ptr = self.local_stack[i];
            if let Some(l) = self.locals.find(&local_ptr) {
                if l.captured {
                    captured = true;
                    capture_reg = capture_reg.min(l.reg);
                }
            }
        }
        if captured {
            unsafe {
                (*self.bytecode).emit_abc(LuauOpcode::LOP_CLOSEUPVALS, capture_reg, 0, 0);
            }
        }
    }
}
