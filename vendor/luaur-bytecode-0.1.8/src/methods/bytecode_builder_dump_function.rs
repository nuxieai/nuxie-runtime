use crate::records::bytecode_builder::BytecodeBuilder;
use alloc::string::String;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub fn dump_function(&self, id: u32) -> String {
        LUAU_ASSERT!(id < self.functions.len() as u32);
        self.functions[id as usize].dump.clone()
    }
}
