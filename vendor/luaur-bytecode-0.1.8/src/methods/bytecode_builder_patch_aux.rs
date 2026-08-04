use crate::records::bytecode_builder::BytecodeBuilder;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub fn patch_aux(&mut self, target_aux: usize, new_value: i32) {
        LUAU_ASSERT!(target_aux < self.insns.len());
        self.insns[target_aux] = new_value as u32;
    }
}
